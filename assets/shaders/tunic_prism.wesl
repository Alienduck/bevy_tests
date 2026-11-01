import bevy_pbr::render::{
    mesh_view_bindings::{view, globals},
    forward_io::VertexOutput,
};

struct PrismParams {
    intensity: f32,
    dispersion: f32,
    fresnel_power: f32,
    edge_sharpness: f32,
    smoke_density: f32,
    smoke_speed: f32,
    mode: f32,
    _pad: f32,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(0)
var<uniform> params: PrismParams;

fn spectral_palette(t: f32) -> vec3<f32> {
    let a = vec3<f32>(0.5, 0.5, 0.5);
    let b = vec3<f32>(0.5, 0.5, 0.5);
    let c = vec3<f32>(1.0, 1.0, 1.0);
    let d = vec3<f32>(0.00, 0.33, 0.67);
    return a + b * cos(6.28318 * (c * t + d));
}

fn hash33(p3: vec3<f32>) -> vec3<f32> {
    var p = fract(p3 * vec3<f32>(0.1031, 0.1030, 0.0973));
    p += dot(p, p.yxz + 33.33);
    return fract((p.xxy + p.yxx) * p.zyx) * 2.0 - 1.0;
}

fn noise3d(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(
            mix(dot(hash33(i + vec3<f32>(0.0, 0.0, 0.0)), f - vec3<f32>(0.0, 0.0, 0.0)),
                dot(hash33(i + vec3<f32>(1.0, 0.0, 0.0)), f - vec3<f32>(1.0, 0.0, 0.0)), u.x),
            mix(dot(hash33(i + vec3<f32>(0.0, 1.0, 0.0)), f - vec3<f32>(0.0, 1.0, 0.0)),
                dot(hash33(i + vec3<f32>(1.0, 1.0, 0.0)), f - vec3<f32>(1.0, 1.0, 0.0)), u.x), u.y),
        mix(
            mix(dot(hash33(i + vec3<f32>(0.0, 0.0, 1.0)), f - vec3<f32>(0.0, 0.0, 1.0)),
                dot(hash33(i + vec3<f32>(1.0, 0.0, 1.0)), f - vec3<f32>(1.0, 0.0, 1.0)), u.x),
            mix(dot(hash33(i + vec3<f32>(0.0, 1.0, 1.0)), f - vec3<f32>(0.0, 1.0, 1.0)),
                dot(hash33(i + vec3<f32>(1.0, 1.0, 1.0)), f - vec3<f32>(1.0, 1.0, 1.0)), u.x), u.y), u.z);
}

fn fbm(p: vec3<f32>) -> f32 {
    var val = 0.0;
    var amp = 0.5;
    var pos = p;
    let rot = mat3x3<f32>(0.00, 0.80, 0.60, -0.80, 0.36, -0.48, -0.60, -0.48, 0.64);
    for (var i = 0; i < 4; i++) {
        val += amp * noise3d(pos);
        pos = rot * pos * 2.03 + vec3<f32>(1.7, 9.2, 3.4);
        amp *= 0.5;
    }
    return val * 0.5 + 0.5;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    let center = vec3<f32>(0.0, 2.5, -4.0);
    let local_pos = in.world_position.xyz - center;
    let view_dir = normalize(view.world_position.xyz - in.world_position.xyz);
    let ray_dir = -view_dir;
    let t = globals.time * params.smoke_speed;

    if (params.mode > 0.5) {
        var accum_light = vec3<f32>(0.0);
        let steps = 12;
        let step_size = 0.085;

        let jitter = fract(sin(dot(in.position.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453);
        var curr_pos = local_pos + ray_dir * jitter * step_size;

        let uv_dist = min(min(in.uv.x, in.uv.y), 1.0 - max(in.uv.x, in.uv.y));
        let ridge_boost = smoothstep(0.35, 0.0, uv_dist);

        for (var i = 0; i < steps; i++) {
            let dist = length(curr_pos);
            let shell_mask = smoothstep(2.15, 1.35, dist) * smoothstep(1.05, 1.45, dist);

            if (shell_mask > 0.001) {
                let flow = vec3<f32>(sin(t * 0.5 + curr_pos.y * 2.0), -t * 0.9, cos(t * 0.4 + curr_pos.x * 2.0));
                let warp = vec3<f32>(
                    fbm(curr_pos * 1.8 + flow * 0.6),
                    fbm(curr_pos * 1.8 - flow * 0.6 + 4.2),
                    fbm(curr_pos * 1.8 + 8.7)
                );
                let n = fbm(curr_pos * 2.4 + warp * 1.3 + flow);
                let density = smoothstep(0.42, 0.85, n + ridge_boost * 0.22) * shell_mask * params.smoke_density;

                if (density > 0.001) {
                    let dir_from_core = normalize(curr_pos);
                    let phase = dot(dir_from_core, vec3<f32>(0.4, 0.7, 0.2)) * 0.8 + n * 0.7 + ridge_boost * 0.3;
                    let emit_col = vec3<f32>(
                        spectral_palette(phase - params.dispersion).r,
                        spectral_palette(phase).g,
                        spectral_palette(phase + params.dispersion).b
                    );

                    let glow = emit_col + vec3<f32>(ridge_boost * 0.45);
                    accum_light += glow * density * step_size * 1.8;
                }
            }
            curr_pos += ray_dir * step_size;
        }

        let grazing = abs(dot(normalize(in.world_normal), view_dir));
        let contour_fade = smoothstep(0.0, 0.45, grazing);
        return vec4<f32>(accum_light * contour_fade * params.intensity, 0.0);
    }

    let normal = normalize(select(-in.world_normal, in.world_normal, is_front));
    let ndotv = abs(dot(normal, view_dir));
    let fresnel = pow(1.0 - ndotv, params.fresnel_power);

    let refract_dir = normalize(refract(ray_dir, normal, 0.72));
    var inner_caustics = vec3<f32>(0.0);
    var p_in = local_pos;
    for (var j = 0; j < 4; j++) {
        p_in += refract_dir * 0.28;
        let cn = fbm(p_in * 2.1 + vec3<f32>(0.0, t * 0.3, 0.0));
        let c_phase = cn * 1.6 + f32(j) * 0.22 + fresnel * 0.5;
        let c_col = vec3<f32>(
            spectral_palette(c_phase - params.dispersion * 1.4).r,
            spectral_palette(c_phase).g,
            spectral_palette(c_phase + params.dispersion * 1.4).b
        );
        inner_caustics += c_col * pow(cn, 2.2) * 0.65;
    }

    let base_phase = fresnel * 1.4 + dot(normal, vec3<f32>(0.4, 0.7, 0.2)) * 0.5;
    let irid = vec3<f32>(
        spectral_palette(base_phase - params.dispersion).r,
        spectral_palette(base_phase).g,
        spectral_palette(base_phase + params.dispersion).b
    );

    let normal_edge = length(fwidth(in.world_normal));
    let uv_edge = pow(1.0 - min(min(in.uv.x, in.uv.y), 1.0 - max(in.uv.x, in.uv.y)) * 2.0, params.edge_sharpness);
    let edge_glow = clamp(normal_edge * 2.5 + uv_edge * 0.8, 0.0, 1.5);

    let core_tint = (vec3<f32>(0.15, 0.04, 0.42) + inner_caustics) * (1.0 - fresnel * 0.4);
    let surface_light = irid * (fresnel * 1.8 + 0.35);
    let ridge_light = (irid + vec3<f32>(0.6)) * edge_glow * 2.2;

    let final_rgb = (core_tint + surface_light + ridge_light) * params.intensity;
    return vec4<f32>(final_rgb, 0.0);
}
