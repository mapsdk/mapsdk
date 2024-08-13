struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) edge: f32,
};

// Map View BindGroup
@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;
@group(0) @binding(1) var<uniform> map_center: vec2<f32>;
@group(0) @binding(2) var<uniform> map_res: f32;

// Params BindGroup
@group(1) @binding(0) var<uniform> z: f32;
@group(1) @binding(1) var<uniform> align: u32; // 0: center 1:left 2:right
@group(1) @binding(2) var<uniform> stroke_width: f32;
@group(1) @binding(3) var<uniform> stroke_color: vec4<f32>;

@vertex
fn vs_main(
    @location(0) vertex_coord: vec2<f32>,
    @location(1) prev_vertex_coord: vec2<f32>,
    @location(2) next_vertex_coord: vec2<f32>,
    @location(3) vertex_type: f32,
    ) -> VertexOutput {
    var p = (vertex_coord - map_center) / map_res;

    let max_edge_scale = 2.0;
    
    let vertex_type_trunc = trunc(vertex_type);
    let vertex_type_frac = fract(vertex_type);

    let start_or_end = sign(vertex_type_trunc - 0.5); // start: -1.0, end: 1.0

    let seg_prev = vertex_coord - prev_vertex_coord;
    let seg_next = next_vertex_coord - vertex_coord;
    let seg = select(seg_prev, seg_next, start_or_end < 0.0);

    let seg_prev_dir = normalize(seg_prev);
    let seg_next_dir = normalize(seg_next);
    let seg_dir = normalize(seg);

    let seg_norm = vec2<f32>(-seg_dir.y, seg_dir.x);
    let seg_length = length(seg) / map_res;

    let join_dir = normalize(seg_prev_dir + seg_next_dir);
    let join_norm = vec2<f32>(-join_dir.y, join_dir.x);

    let angle = select(0.0, acos(clamp(dot(seg_prev_dir, seg_next_dir), -1.0, 1.0)), length(seg_prev) * length(seg_next) > 0.0); // 0 ~ PI
    let angle_sign = sign(seg_prev_dir.x * seg_next_dir.y - seg_prev_dir.y * seg_next_dir.x);

    let edge_width = select(stroke_width, stroke_width / 2.0, align == 0);
    let align_sign = select(-1.0, 1.0, align == 2);

    if vertex_type_frac < 0.25 { // straight line center
        if align == 0 || (align_sign * angle_sign > 0.0) {
            p -= seg_dir * min(seg_length, edge_width * tan(angle / 2.0)) * start_or_end;
        }
    }

    if vertex_type_frac < 0.25 { // straight line
        let edge_sign = sign(vertex_type_frac - 0.15);
        if align == 0 {
            p += seg_norm * edge_width * edge_sign;
        } else {
            if vertex_type_frac > 0.15 {
                p += seg_norm * edge_width * edge_sign * align_sign;
            }
        }
    } else if vertex_type_frac > 0.25 && vertex_type_frac < 0.35 { // join edge
        if align == 0 || (align > 0 && align_sign * angle_sign < 0.0) {
            let edge_scale = 1.0 / cos(angle / 2.0);
            if edge_scale < max_edge_scale {
                p -= join_norm * edge_width * edge_scale * angle_sign;
            } else {
                let w = min(seg_length, edge_width * max_edge_scale);
                p -= join_norm * w * angle_sign;
                p -= join_dir * sin(angle / 2.0) * (edge_width * edge_scale - w) / edge_scale * start_or_end;
            }
        }
    }

    let position = view_proj * vec4<f32>(p.xy, z / map_res, 1.0);

    var edge = 0.0;
    if align == 0 {
        if vertex_type_frac < 0.25 {
            edge = 1.0 * sign(vertex_type_frac - 0.15);
        } else if vertex_type_frac > 0.25 {
            edge = -1.0 * angle_sign;
        }
    } else {
        if vertex_type_frac > 0.15 && vertex_type_frac < 0.25 {
            edge = 1.0;
        } else if vertex_type_frac > 0.25 {
            edge = select(1.0, 0.0, align_sign * angle_sign > 0.0);
        }
    }

    return VertexOutput(position, stroke_color, edge);
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = smoothstep(0.0, 1.0, 2.0 * (1.0 - abs(vertex.edge)));
    
    return vec4(vertex.color.xyz, alpha * vertex.color.w);
}
