struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) v_tex_translation: vec2<f32>,
    @location(1) layer: u32,
    @location(2) color_picking_override: vec4<f32>,
    @location(3) enable_color_picking_override: u32,
    @location(4) enable_highlight: u32,
    @location(5) highlight_color: vec4<f32>
 };

struct Uniforms {
    model_trans: mat4x4<f32>,
    camera_view: mat4x4<f32>
}

struct PickingData {
    enabled: u32
}



@group(0)
@binding(0)
var<uniform> r_data: Uniforms;

@vertex
fn vs_main(
    @location(0) a_position : vec3<f32>,
    @location(1) a_tex_translation : vec2<f32>,
    @location(2) layer: u32,
    @location(3) depth: f32,
    @location(4) color_picking_override: vec4<f32>,
    @location(5) enable_color_picking_override: u32,
    @location(6) enable_highlight: u32,
    @location(7) highlight_color: vec4<f32>
) ->  VertexOutput {
    var result: VertexOutput;
    let world_position = r_data.model_trans * vec4<f32>(a_position, 1.0);
    var clip_position = r_data.camera_view * world_position;
    clip_position.z += depth;
    result.position = clip_position;
    result.v_tex_translation = a_tex_translation;
    result.layer = u32(layer);
    result.color_picking_override = color_picking_override;
    result.enable_color_picking_override = u32(enable_color_picking_override);
    result.enable_highlight = u32(enable_highlight);
    result.highlight_color = highlight_color;
    return result;
}

@group(1)
@binding(0)
var t_diffuse: texture_2d_array<f32>;

@group(1)
@binding(1)
var s_diffuse: sampler;

@group(2)
@binding(0)
var<uniform> picking_data: PickingData;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
   let depth = vertex.position.z / vertex.position.w;
   let color = textureSample(t_diffuse, s_diffuse, vertex.v_tex_translation, vertex.layer);

   if (color.a < 0.0001) {
       discard;
   }

   if (picking_data.enabled > 0 && vertex.enable_color_picking_override > 0) {
       return vertex.color_picking_override;
   }

   if(vertex.enable_highlight > 0){
   let final_color = vec4<f32>(
       mix(color.rgb, color.rgb + vertex.highlight_color.rgb * vertex.highlight_color.a, vertex.highlight_color.a),
       color.a
    );
    return final_color;
   }

   return color;
}