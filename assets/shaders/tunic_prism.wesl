import bevy_pbr::render::{
    mesh_view_bindings::view,
    forward_io::VertexOutput,
};

struct PrismParams {
    intensity: f32,
    edge_thickness_px: f32,
    smear_length: f32,
    dispersion: f32,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(0)
var<uniform> params: PrismParams;

fn neon_spectrum(t: f32) -> vec3<f32> {
    let col = 0.52 + 0.48 * cos(6.28318 * (t + vec3<f32>(0.0, 0.31, 0.65)));
    return pow(max(col, vec3<f32>(0.0)), vec3<f32>(1.35)) * 1.45;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    let view_dir = normalize(view.world_position.xyz - in.world_position.xyz);
    let normal = normalize(select(-in.world_normal, in.world_normal, is_front));
    let ndotv = abs(dot(normal, view_dir));
    let fresnel = pow(1.0 - ndotv, 1.8);

    let b = in.color.rgb;
    let fw = max(fwidth(b), vec3<f32>(0.0001));
    let px = b / fw;
    let min_px = min(min(px.x, px.y), px.z);

    let refract_dir = refract(-view_dir, normal, select(0.62, 0.74, is_front));
    let astig_weights = vec3<f32>(
        abs(dot(refract_dir, vec3<f32>(0.88, -0.35, 0.32))),
        abs(dot(refract_dir, vec3<f32>(-0.38, 0.85, -0.36))),
        abs(dot(refract_dir, vec3<f32>(0.28, 0.40, 0.87)))
    );

    let base_phase = dot(normal, vec3<f32>(0.55, 0.72, -0.42)) * 0.5 + fresnel * 0.45;
    var total_smear = vec3<f32>(0.0);

    for (var k = 0u; k < 3u; k++) {
        let d = b[k];
        let along = b[(k + 1u) % 3u] - b[(k + 2u) % 3u];
        let corner_prox = pow(abs(along), 1.6);

        let wave = 0.55 + 0.45 * sin(along * 3.6 + f32(k) * 2.09 + dot(normal, view_dir) * 4.0);
        let reach = clamp(
            (0.02 + pow(astig_weights[k], 1.5) * params.smear_length * (0.5 + fresnel * 0.9 + corner_prox * 0.6)) * wave,
            0.015,
            0.34
        );

        let x_r = d / (reach * (1.0 + params.dispersion * 0.45));
        let x_g = d / reach;
        let x_b = d / max(reach * (1.0 - params.dispersion * 0.35), 0.008);

        let env_r = pow(clamp(1.0 - x_r, 0.0, 1.0), 2.0);
        let env_g = pow(clamp(1.0 - x_g, 0.0, 1.0), 2.2);
        let env_b = pow(clamp(1.0 - x_b, 0.0, 1.0), 2.4);

        let ghost_r = exp(-pow((x_r - 0.42) / 0.11, 2.0)) * 0.65;
        let ghost_g = exp(-pow((x_g - 0.32) / 0.09, 2.0)) * 0.55;
        let ghost_b = exp(-pow((x_b - 0.22) / 0.08, 2.0)) * 0.75;

        let local_phase = base_phase + f32(k) * 0.28 + along * 0.22 + x_g * params.dispersion;
        let spec = neon_spectrum(local_phase);

        let edge_smear = vec3<f32>(
            (env_r + ghost_r) * spec.r,
            (env_g + ghost_g) * spec.g,
            (env_b + ghost_b) * spec.b
        ) * (0.65 + corner_prox * 1.15);

        total_smear += edge_smear;
    }

    let max_b = max(max(b.x, b.y), b.z);
    let vertex_flare = pow( smoothstep(0.68, 0.99, max_b), 2.0 ) * 2.2;
    let vertex_col = neon_spectrum(base_phase + 0.15) * vertex_flare;

    let wire_core = 1.0 - smoothstep(0.0, params.edge_thickness_px, min_px);
    let wire_fringe = 1.0 - smoothstep(params.edge_thickness_px * 0.8, params.edge_thickness_px * 3.2, min_px);
    let edge_spec = neon_spectrum(base_phase + (b.x - b.y) * 0.35);
    let sharp_edge = vec3<f32>(1.0, 0.98, 1.0) * wire_core * 2.4 + edge_spec * wire_fringe * 1.3;

    let face_weight = select(0.55, 1.0, is_front);
    let final_rgb = (sharp_edge + total_smear * 1.25 + vertex_col) * face_weight * params.intensity;

    return vec4<f32>(final_rgb, 0.0);
}
