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
    core_opacity: f32,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(0)
var<uniform> params: PrismParams;

fn tunic_spectrum(t: f32) -> vec3<f32> {
    let a = vec3<f32>(0.52, 0.45, 0.58);
    let b = vec3<f32>(0.48, 0.55, 0.45);
    let c = vec3<f32>(1.0, 1.0, 1.0);
    let d = vec3<f32>(0.00, 0.30, 0.64);
    let col = a + b * cos(6.28318 * (c * t + d));
    return pow(max(col, vec3<f32>(0.0)), vec3<f32>(1.25)) * 1.35;
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
    for (var i = 0; i < 3; i++) {
        val += amp * noise3d(pos);
        pos = rot * pos * 2.02 + vec3<f32>(1.7, 9.2, 3.4);
        amp *= 0.5;
    }
    return val * 0.5 + 0.5;
}

fn ridged_caustic(p: vec3<f32>) -> f32 {
    let n1 = 1.0 - abs(noise3d(p));
    let n2 = 1.0 - abs(noise3d(p * 2.05 + vec3<f32>(3.1, -2.4, 5.7)));
    return pow(clamp(n1 * n2, 0.0, 1.0), 2.4);
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

    let bary = in.color.rgb;
    let d_edge = min(min(bary.x, bary.y), bary.z);

    if (params.mode > 0.5 && params.mode < 1.5) {
        var accum_light = vec3<f32>(0.0);
        let steps = 12;
        let step_size = 0.055;

        let jitter = fract(sin(dot(in.position.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453);
        var curr_pos = local_pos + ray_dir * jitter * step_size;
        let edge_proximity = smoothstep(0.32, 0.0, d_edge);

        for (var i = 0; i < steps; i++) {
            let dist = length(curr_pos);
            let dir = curr_pos / max(dist, 0.001);

            let advected_pos = curr_pos * 2.1 - dir * (t * 1.35) - vec3<f32>(0.0, t * 0.65, 0.0);

            let curl_warp = vec3<f32>(
                fbm(advected_pos + vec3<f32>(0.0, t * 0.4, 0.0)),
                fbm(advected_pos + vec3<f32>(4.3, -t * 0.35, 1.9)),
                fbm(advected_pos + vec3<f32>(8.1, 2.7, t * 0.4))
            ) - 0.5;

            let warped_p = advected_pos + curl_warp * 1.65;
            let disp = params.dispersion * 1.1;

            let n_r = fbm(warped_p + vec3<f32>(disp, -disp * 0.5, 0.0));
            let n_g = fbm(warped_p);
            let n_b = fbm(warped_p - vec3<f32>(disp, -disp * 0.5, 0.0));

            let dynamic_outer = 1.62 + n_g * 0.52 + edge_proximity * 0.18;
            let skin_mask = smoothstep(dynamic_outer, 1.28, dist) * smoothstep(1.02, 1.32, dist);

            if (skin_mask > 0.001) {
                let dissipation = smoothstep(1.25, 2.1, dist) * 0.28;
                let threshold = 0.36 + dissipation - edge_proximity * 0.16;

                let w_r = smoothstep(threshold, threshold + 0.35, n_r);
                let w_g = smoothstep(threshold, threshold + 0.35, n_g);
                let w_b = smoothstep(threshold, threshold + 0.35, n_b);

                let phase = dot(dir, vec3<f32>(0.55, 0.65, -0.35)) * 0.75 + n_g * 0.8 - t * 0.15;

                let col = vec3<f32>(
                    tunic_spectrum(phase - params.dispersion).r * w_r,
                    tunic_spectrum(phase).g * w_g,
                    tunic_spectrum(phase + params.dispersion).b * w_b
                );

                let flare = pow(edge_proximity, 2.0) * tunic_spectrum(phase + 0.12) * w_g * 1.6;
                accum_light += (col + flare) * skin_mask * params.smoke_density * step_size;
            }
            curr_pos += ray_dir * step_size;
        }

        let grazing = abs(dot(normalize(in.world_normal), view_dir));
        let contour_fade = smoothstep(0.0, 0.38, grazing);
        return vec4<f32>(accum_light * contour_fade * params.intensity, 0.0);
    }

    let normal = normalize(select(-in.world_normal, in.world_normal, is_front));
    let ndotv = abs(dot(normal, view_dir));
    let fresnel = pow(1.0 - ndotv, params.fresnel_power);

    let chroma_shift = params.dispersion * 0.065;
    let edge_r = smoothstep(params.edge_sharpness * 1.4, 0.0, abs(d_edge - chroma_shift));
    let edge_g = smoothstep(params.edge_sharpness, 0.0, d_edge);
    let edge_b = smoothstep(params.edge_sharpness * 1.8, 0.0, abs(d_edge - chroma_shift * 2.1));
    let laser_core = smoothstep(params.edge_sharpness * 0.28, 0.0, d_edge);

    let facet_phase = dot(normal, vec3<f32>(0.45, 0.72, -0.35)) * 0.65 + fresnel * 0.85;
    let spec_r = tunic_spectrum(facet_phase - params.dispersion).r;
    let spec_g = tunic_spectrum(facet_phase).g;
    let spec_b = tunic_spectrum(facet_phase + params.dispersion).b;
    let facet_irid = vec3<f32>(spec_r, spec_g, spec_b);

    let edge_aberration = vec3<f32>(
        edge_r * max(spec_r, 0.65),
        edge_g * max(spec_g, 0.65),
        edge_b * max(spec_b, 0.75)
    ) * 1.8 + vec3<f32>(1.0, 0.96, 1.0) * laser_core * 2.6;

    if (params.mode > 1.5) {
        let ghost_glow = edge_aberration * (fresnel * 1.2 + 0.3) * params.intensity;
        return vec4<f32>(ghost_glow, 0.0);
    }

    let refract_r = normalize(refract(ray_dir, normal, 0.68));
    let refract_g = normalize(refract(ray_dir, normal, 0.73));
    let refract_b = normalize(refract(ray_dir, normal, 0.78));

    let p_base = local_pos * 1.45 + vec3<f32>(0.0, t * 0.15, 0.0);
    let c_r = ridged_caustic(p_base + refract_r * 0.55);
    let c_g = ridged_caustic(p_base + refract_g * 0.55);
    let c_b = ridged_caustic(p_base + refract_b * 0.55);

    let caustic_ribbons = vec3<f32>(
        c_r * tunic_spectrum(facet_phase - 0.15).r,
        c_g * tunic_spectrum(facet_phase + 0.05).g,
        c_b * tunic_spectrum(facet_phase + 0.25).b
    ) * 2.4;

    let corner_glow = pow(1.0 - max(max(bary.x, bary.y), bary.z), 3.0);
    let vertex_flare = tunic_spectrum(facet_phase + 0.2) * corner_glow * 1.1;

    let deep_indigo = mix(
        vec3<f32>(0.05, 0.015, 0.22),
        vec3<f32>(0.18, 0.04, 0.48),
         clamp(dot(normal, vec3<f32>(0.0, 1.0, 0.0)) * 0.5 + 0.5, 0.0, 1.0)
    );

    let surface_wash = facet_irid * pow(fresnel, 1.3) * 0.95;
    let total_light = (deep_indigo + caustic_ribbons + surface_wash + vertex_flare + edge_aberration) * params.intensity;

    let alpha = clamp(params.core_opacity * (1.0 - fresnel * 0.55), 0.0, 1.0);
    return vec4<f32>(total_light, alpha);
}
