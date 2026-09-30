// WGSL compute shader: real-time ADM shift field & boundary relabeling
// visualization over a 4096 x 4096 metric grid.
struct MetricField {
    g00: f32,
    g01: f32,
    g02: f32,
    g03: f32,
    shift_x: f32,
    shift_y: f32,
    shift_z: f32,
    ds2: f32,
};

@group(0) @binding(0) var<storage, read> inputFields: array<MetricField>;
@group(0) @binding(1) var<storage, read_write> outputRenderBuffer: array<vec4<f32>>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let width: u32 = 4096u;
    let index = global_id.y * width + global_id.x;

    let field = inputFields[index];

    let shift_mag = sqrt(
        field.shift_x * field.shift_x
        + field.shift_y * field.shift_y
        + field.shift_z * field.shift_z
    );

    var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    if (field.ds2 > 0.0) {
        // Causal violation warning
        color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    } else {
        // Nominal metric visualization mapped by shift magnitude
        color = vec4<f32>(0.0, shift_mag * 0.5, 1.0 - shift_mag * 0.2, 1.0);
    }

    outputRenderBuffer[index] = color;
}
