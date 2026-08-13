//! CAD/EDA export synthesis for the SHBT array and sapphire waveguide.
//!
//! - GDSII mask generation: 8×8 emitter array with 50 μm pitch, substrate,
//!   airbridge span and niobium trace layers.
//! - STEP solid modelling: ISO 10303-21 B-Rep export of a rectangular sapphire
//!   waveguide so that interface dimensions can be checked against the nominal
//!   acoustic impedance target.

use pyo3::prelude::*;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Pitch between adjacent SHBT emitters (μm).
pub const EMITTER_PITCH_UM: f64 = 50.0;
/// Substrate/InP layer square dimension (μm).
pub const SUBSTRATE_SIZE_UM: f64 = 350.0;
/// Airbridge span width and height (μm).
pub const AIRBRIDGE_WIDTH_UM: f64 = 1.5;
pub const AIRBRIDGE_HEIGHT_UM: f64 = 5.0;
/// Niobium trace width (nm).
pub const TRACE_WIDTH_NM: f64 = 300.0;
/// Minimum resolvable feature size for standard electron-beam lithography (nm).
pub const MIN_EBEAM_RESOLUTION_NM: f64 = 50.0;
/// Nominal acoustic impedance for the matching layer (MRayl).
pub const NOMINAL_IMPEDANCE_MRAYL: f64 = 1.1512;

/// Convert micrometers to picometers (database units for GDSII).
fn um_to_pm(um: f64) -> i32 {
    (um * 1_000_000.0).round() as i32
}

/// Encode a positive or negative f64 into the GDSII 8-byte real format.
///
/// GDSII real numbers use base-16 exponent (bias 64) and a 56-bit mantissa
/// interpreted as a fractional value with the hexadecimal point to the left of
/// the most-significant digit.  This routine normalises the value, packs the
/// mantissa into 7 bytes, and stores the exponent in the first byte.
fn gds_real8(value: f64) -> [u8; 8] {
    if value == 0.0 {
        return [0; 8];
    }
    let sign_byte = if value.is_sign_negative() { 0x80u8 } else { 0x00u8 };
    let mut a = value.abs();

    // Normalise a into [1/16, 1)
    let mut exp: i32 = 0;
    while a >= 1.0 {
        a /= 16.0;
        exp += 1;
    }
    while a < 1.0 / 16.0 {
        a *= 16.0;
        exp -= 1;
    }

    let scale = (16.0_f64).powi(14);
    let mut mantissa = (a * scale).round() as u64;
    if mantissa >= (1u64 << 56) {
        mantissa = (1u64 << 56) - 1;
    }

    let mut bytes = [0u8; 8];
    bytes[0] = sign_byte | ((exp + 64) as u8);
    for i in 0..7 {
        bytes[7 - i] = ((mantissa >> (8 * i)) & 0xff) as u8;
    }
    bytes
}

fn write_u16_be(w: &mut impl Write, v: u16) -> std::io::Result<()> {
    w.write_all(&v.to_be_bytes())
}

fn write_i16_be(w: &mut impl Write, v: i16) -> std::io::Result<()> {
    w.write_all(&v.to_be_bytes())
}

fn write_i32_be(w: &mut impl Write, v: i32) -> std::io::Result<()> {
    w.write_all(&v.to_be_bytes())
}

fn write_padded_string(w: &mut impl Write, s: &str) -> std::io::Result<()> {
    let mut buf = s.as_bytes().to_vec();
    if buf.len() % 2 == 0 {
        buf.push(0);
    } else {
        buf.push(0);
        buf.push(0);
    }
    w.write_all(&buf)
}

fn write_record_header(
    w: &mut impl Write,
    length: u16,
    record_type: u8,
    data_type: u8,
) -> std::io::Result<()> {
    write_u16_be(w, length)?;
    w.write_all(&[record_type, data_type])
}

fn write_no_data_record(w: &mut impl Write, record_type: u8) -> std::io::Result<()> {
    write_record_header(w, 4, record_type, 0x00)
}

fn write_i16_record(w: &mut impl Write, record_type: u8, value: i16) -> std::io::Result<()> {
    write_record_header(w, 6, record_type, 0x02)?;
    write_i16_be(w, value)
}

fn write_string_record(w: &mut impl Write, record_type: u8, s: &str) -> std::io::Result<()> {
    let bytes = s.as_bytes();
    let payload = if bytes.len() % 2 == 0 {
        bytes.len() + 2 // trailing null + padding
    } else {
        bytes.len() + 1 // trailing null only
    };
    write_record_header(w, (4 + payload) as u16, record_type, 0x06)?;
    write_padded_string(w, s)
}

fn write_real_pair_record(
    w: &mut impl Write,
    record_type: u8,
    a: f64,
    b: f64,
) -> std::io::Result<()> {
    write_record_header(w, 20, record_type, 0x05)?;
    w.write_all(&gds_real8(a))?;
    w.write_all(&gds_real8(b))
}

/// Write a rectangular boundary polygon for a GDSII layer.
fn write_boundary(
    w: &mut impl Write,
    layer: i16,
    datatype: i16,
    lower_left: (i32, i32),
    upper_right: (i32, i32),
) -> std::io::Result<()> {
    let (x0, y0) = lower_left;
    let (x1, y1) = upper_right;
    // GDSII boundary: at least 4 points and first point repeated.
    let points: [(i32, i32); 5] = [(x0, y0), (x1, y0), (x1, y1), (x0, y1), (x0, y0)];
    let n = points.len() as u16;

    write_no_data_record(w, 0x08)?; // BOUNDARY
    write_i16_record(w, 0x0D, layer)?; // LAYER
    write_i16_record(w, 0x0E, datatype)?; // DATATYPE
    write_record_header(w, 4 + 4 * 2 * n, 0x10, 0x03)?; // XY
    for (x, y) in points {
        write_i32_be(w, x)?;
        write_i32_be(w, y)?;
    }
    write_no_data_record(w, 0x11)?; // ENDEL
    Ok(())
}

/// GDSII mask exporter for the SHBT emitter array.
#[pyclass(name = "GdsiiMaskExporter")]
#[derive(Clone, Debug)]
pub struct GdsiiMaskExporter;

impl GdsiiMaskExporter {
    pub fn new() -> Self {
        Self
    }

    /// Validate the mask against a simple electron-beam DRC.
    ///
    /// Checks that all drawn features are at least `MIN_EBEAM_RESOLUTION_NM` in
    /// width and height.  Returns `(ok, Vec<violation messages>)`.
    pub fn validate_drc_impl(&self) -> (bool, Vec<String>) {
        let mut violations = Vec::new();

        let features: [(&str, f64, f64); 3] = [
            (
                "SUBSTRATE_INP",
                SUBSTRATE_SIZE_UM * 1000.0,
                SUBSTRATE_SIZE_UM * 1000.0,
            ),
            (
                "AIRBRIDGE_SPAN",
                AIRBRIDGE_WIDTH_UM * 1000.0,
                AIRBRIDGE_HEIGHT_UM * 1000.0,
            ),
            ("MET_NB_TRACE", TRACE_WIDTH_NM, AIRBRIDGE_HEIGHT_UM * 1000.0),
        ];

        for (name, width_nm, height_nm) in features {
            if width_nm < MIN_EBEAM_RESOLUTION_NM {
                violations.push(format!(
                    "DRC violation: {} width {:.3} nm is below {:.3} nm e-beam resolution",
                    name, width_nm, MIN_EBEAM_RESOLUTION_NM
                ));
            }
            if height_nm < MIN_EBEAM_RESOLUTION_NM {
                violations.push(format!(
                    "DRC violation: {} height {:.3} nm is below {:.3} nm e-beam resolution",
                    name, height_nm, MIN_EBEAM_RESOLUTION_NM
                ));
            }
        }

        (violations.is_empty(), violations)
    }

    /// Export a minimal GDSII stream file for an 8×8 SHBT array.
    pub fn export_array_impl<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut f = File::create(path)?;

        // GDSII header: version 600
        write_i16_record(&mut f, 0x00, 600)?;

        // BGNLIB: last modification/access = all zeros
        write_record_header(&mut f, 28, 0x01, 0x02)?;
        for _ in 0..12 {
            write_i16_be(&mut f, 0)?;
        }

        write_string_record(&mut f, 0x02, "shbt_mask.gds")?;
        // 1 database unit = 1 pm = 1.0E-12 m
        write_real_pair_record(&mut f, 0x03, 1.0, 1.0e-12)?;

        // BGNSTR
        write_record_header(&mut f, 28, 0x05, 0x02)?;
        for _ in 0..12 {
            write_i16_be(&mut f, 0)?;
        }
        write_string_record(&mut f, 0x06, "shbt_array")?;

        // Layer 10: SUBSTRATE_INP (350 μm square)
        let sub_half = um_to_pm(SUBSTRATE_SIZE_UM);
        write_boundary(&mut f, 10, 0, (0, 0), (sub_half, sub_half))?;

        // Emitter array layers
        let pitch_pm = um_to_pm(EMITTER_PITCH_UM);
        let air_half_x = um_to_pm(AIRBRIDGE_WIDTH_UM / 2.0);
        let air_half_y = um_to_pm(AIRBRIDGE_HEIGHT_UM / 2.0);
        let trace_half_x = um_to_pm(TRACE_WIDTH_NM / 1000.0 / 2.0);
        let trace_half_y = air_half_y;

        for u in 0..8 {
            for v in 0..8 {
                let cx = u * pitch_pm + pitch_pm / 2;
                let cy = v * pitch_pm + pitch_pm / 2;

                // Layer 20: AIRBRIDGE_SPAN (1.5 × 5.0 μm)
                write_boundary(
                    &mut f,
                    20,
                    0,
                    (cx - air_half_x, cy - air_half_y),
                    (cx + air_half_x, cy + air_half_y),
                )?;

                // Layer 25: MET_NB_TRACE (300 nm × 5.0 μm)
                write_boundary(
                    &mut f,
                    25,
                    0,
                    (cx - trace_half_x, cy - trace_half_y),
                    (cx + trace_half_x, cy + trace_half_y),
                )?;
            }
        }

        write_no_data_record(&mut f, 0x07)?; // ENDSTR
        write_no_data_record(&mut f, 0x04)?; // ENDLIB
        f.flush()
    }
}

#[pymethods]
impl GdsiiMaskExporter {
    #[new]
    fn py_new() -> Self {
        Self::new()
    }

    /// Export the SHBT 8×8 array mask to `path`.
    fn export_array(&self, path: &str) -> PyResult<()> {
        self.export_array_impl(path).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("GDSII export failed: {}", e))
        })
    }

    /// Run the electron-beam DRC check and return `(ok, violations)`.
    fn validate_drc(&self) -> (bool, Vec<String>) {
        self.validate_drc_impl()
    }

    fn nominal_impedance_mrayl(&self) -> f64 {
        NOMINAL_IMPEDANCE_MRAYL
    }
}

/// STEP (ISO 10303-21) solid-model exporter.
#[pyclass(name = "StepSolidModel")]
#[derive(Clone, Debug)]
pub struct StepSolidModel;

impl StepSolidModel {
    pub fn new() -> Self {
        Self
    }

    /// Export a rectangular sapphire waveguide as a STEP B-Rep file.
    pub fn export_waveguide_impl<P: AsRef<Path>>(
        &self,
        path: P,
        length_m: f64,
        width_m: f64,
        height_m: f64,
    ) -> std::io::Result<()> {
        let mut s = String::new();
        s.push_str("ISO-10303-21;\n");
        s.push_str("HEADER;\n");
        s.push_str("FILE_DESCRIPTION(('sapphire waveguide B-Rep'), '2;1');\n");
        s.push_str("FILE_NAME('sapphire_waveguide.step', '2026-08-13T00:00:00', (''), (''), '', '', '');\n");
        s.push_str("FILE_SCHEMA(('AUTOMOTIVE_DESIGN { 1 0 10303 214 1 1 1 1 }'));\n");
        s.push_str("ENDSEC;\n");
        s.push_str("DATA;\n");

        let mut id = 1usize;
        let mut next_id = || {
            let v = id;
            id += 1;
            v
        };

        let l = length_m;
        let w = width_m;
        let h = height_m;

        // Vertices
        let verts: [(f64, f64, f64); 8] = [
            (0.0, 0.0, 0.0),
            (l, 0.0, 0.0),
            (l, w, 0.0),
            (0.0, w, 0.0),
            (0.0, 0.0, h),
            (l, 0.0, h),
            (l, w, h),
            (0.0, w, h),
        ];

        let mut point_ids = [0usize; 8];
        for (i, (x, y, z)) in verts.iter().enumerate() {
            let pid = next_id();
            point_ids[i] = pid;
            s.push_str(&format!(
                "#{} = CARTESIAN_POINT('v{}', ({:.15}, {:.15}, {:.15}));\n",
                pid, i, x, y, z
            ));
        }

        let mut vertex_ids = [0usize; 8];
        for i in 0..8 {
            let vid = next_id();
            vertex_ids[i] = vid;
            s.push_str(&format!(
                "#{} = VERTEX_POINT('V{}', #{});\n",
                vid, i, point_ids[i]
            ));
        }

        // Edge curves (12 edges, each direction low->high index)
        let edges: [(usize, usize); 12] = [
            (0, 1), // +x, bottom
            (1, 2), // +y, bottom
            (2, 3), // -x, bottom
            (3, 0), // -y, bottom
            (0, 4), // +z
            (1, 5), // +z
            (2, 6), // +z
            (3, 7), // +z
            (4, 5), // +x, top
            (5, 6), // +y, top
            (6, 7), // -x, top
            (7, 4), // -y, top
        ];

        let edge_dir: [(f64, f64, f64); 12] = [
            (1.0, 0.0, 0.0),
            (0.0, 1.0, 0.0),
            (-1.0, 0.0, 0.0),
            (0.0, -1.0, 0.0),
            (0.0, 0.0, 1.0),
            (0.0, 0.0, 1.0),
            (0.0, 0.0, 1.0),
            (0.0, 0.0, 1.0),
            (1.0, 0.0, 0.0),
            (0.0, 1.0, 0.0),
            (-1.0, 0.0, 0.0),
            (0.0, -1.0, 0.0),
        ];

        let mut edge_curve_ids = [0usize; 12];
        for (ei, ((a, b), (dx, dy, dz))) in edges.iter().zip(edge_dir.iter()).enumerate() {
            let line_pid = next_id();
            let dir_id = next_id();
            let vec_id = next_id();
            let edge_id = next_id();
            edge_curve_ids[ei] = edge_id;

            s.push_str(&format!(
                "#{} = CARTESIAN_POINT('e{}_origin', ({:.15}, {:.15}, {:.15}));\n",
                line_pid, ei, verts[*a].0, verts[*a].1, verts[*a].2
            ));
            s.push_str(&format!(
                "#{} = DIRECTION('e{}_dir', ({:.15}, {:.15}, {:.15}));\n",
                dir_id, ei, dx, dy, dz
            ));
            s.push_str(&format!(
                "#{} = VECTOR('e{}_vec', #{}, 1.0);\n",
                vec_id, ei, dir_id
            ));
            s.push_str(&format!(
                "#{} = LINE('e{}_line', #{}, #{});\n",
                line_pid, ei, line_pid, vec_id
            ));
            s.push_str(&format!(
                "#{} = EDGE_CURVE('e{}', #{}, #{}, #{}, .T.);\n",
                edge_id, ei, vertex_ids[*a], vertex_ids[*b], line_pid
            ));
        }

        // Face loops: list of (edge_index, orientation)
        let faces: [[(usize, bool); 4]; 6] = [
            [(3, false), (2, false), (1, false), (0, false)], // bottom
            [(8, true), (9, true), (10, true), (11, true)],    // top
            [(0, true), (5, true), (8, false), (4, false)],    // front
            [(7, true), (10, false), (6, false), (2, true)],   // back
            [(4, true), (11, false), (7, false), (3, true)],  // left
            [(1, true), (6, true), (9, false), (5, false)],    // right
        ];

        let face_origins: [(f64, f64, f64); 6] = [
            (0.0, 0.0, 0.0),
            (0.0, 0.0, h),
            (0.0, 0.0, 0.0),
            (0.0, w, 0.0),
            (0.0, 0.0, 0.0),
            (l, 0.0, 0.0),
        ];
        let face_normals: [(f64, f64, f64); 6] = [
            (0.0, 0.0, -1.0),
            (0.0, 0.0, 1.0),
            (0.0, -1.0, 0.0),
            (0.0, 1.0, 0.0),
            (-1.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
        ];
        let face_refs: [(f64, f64, f64); 6] = [
            (1.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
            (0.0, 1.0, 0.0),
            (0.0, 1.0, 0.0),
        ];

        let mut face_ids = [0usize; 6];
        for fi in 0..6 {
            let mut oriented_edge_ids = Vec::with_capacity(4);
            for (ei, orient) in faces[fi] {
                let oe_id = next_id();
                let orient_str = if orient { ".T." } else { ".F." };
                s.push_str(&format!(
                    "#{} = ORIENTED_EDGE('', *, *, #{}, {});\n",
                    oe_id, edge_curve_ids[ei], orient_str
                ));
                oriented_edge_ids.push(oe_id);
            }

            let loop_id = next_id();
            let edge_list = oriented_edge_ids
                .iter()
                .map(|x| format!("#{}", x))
                .collect::<Vec<_>>()
                .join(", ");
            s.push_str(&format!("#{} = EDGE_LOOP('', ({}));\n", loop_id, edge_list));

            let bound_id = next_id();
            s.push_str(&format!(
                "#{} = FACE_OUTER_BOUND('', #{}, .T.);\n",
                bound_id, loop_id
            ));

            let origin_id = next_id();
            let normal_id = next_id();
            let ref_id = next_id();
            let axis_id = next_id();
            let plane_id = next_id();

            let (ox, oy, oz) = face_origins[fi];
            let (nx, ny, nz) = face_normals[fi];
            let (rx, ry, rz) = face_refs[fi];

            s.push_str(&format!(
                "#{} = CARTESIAN_POINT('f{}_origin', ({:.15}, {:.15}, {:.15}));\n",
                origin_id, fi, ox, oy, oz
            ));
            s.push_str(&format!(
                "#{} = DIRECTION('f{}_normal', ({:.15}, {:.15}, {:.15}));\n",
                normal_id, fi, nx, ny, nz
            ));
            s.push_str(&format!(
                "#{} = DIRECTION('f{}_ref', ({:.15}, {:.15}, {:.15}));\n",
                ref_id, fi, rx, ry, rz
            ));
            s.push_str(&format!(
                "#{} = AXIS2_PLACEMENT_3D('f{}_axis', #{}, #{}, #{});\n",
                axis_id, fi, origin_id, normal_id, ref_id
            ));
            s.push_str(&format!("#{} = PLANE('f{}', #{});\n", plane_id, fi, axis_id));

            let face_id = next_id();
            face_ids[fi] = face_id;
            s.push_str(&format!(
                "#{} = ADVANCED_FACE('f{}', (#{}), #{}, .T.);\n",
                face_id, fi, bound_id, plane_id
            ));
        }

        let shell_id = next_id();
        let face_refs_str = face_ids
            .iter()
            .map(|x| format!("#{}", x))
            .collect::<Vec<_>>()
            .join(", ");
        s.push_str(&format!(
            "#{} = CLOSED_SHELL('waveguide_shell', ({}));\n",
            shell_id, face_refs_str
        ));

        let solid_id = next_id();
        s.push_str(&format!(
            "#{} = MANIFOLD_SOLID_BREP('sapphire_waveguide', #{});\n",
            solid_id, shell_id
        ));

        s.push_str("ENDSEC;\n");
        s.push_str("END-ISO-10303-21;\n");

        let mut f = File::create(path)?;
        f.write_all(s.as_bytes())?;
        f.flush()
    }
}

#[pymethods]
impl StepSolidModel {
    #[new]
    fn py_new() -> Self {
        Self::new()
    }

    /// Export a STEP B-Rep of a rectangular sapphire waveguide.
    fn export_waveguide(
        &self,
        path: &str,
        length_m: f64,
        width_m: f64,
        height_m: f64,
    ) -> PyResult<()> {
        self.export_waveguide_impl(path, length_m, width_m, height_m)
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                    "STEP export failed: {}",
                    e
                ))
            })
    }

    /// Nominal acoustic impedance the waveguide interface targets (MRayl).
    fn nominal_impedance_mrayl(&self) -> f64 {
        NOMINAL_IMPEDANCE_MRAYL
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn gdsii_exporter_creates_file_with_header() {
        let tmp = std::env::temp_dir().join("shbt_test.gds");
        let exporter = GdsiiMaskExporter::new();
        exporter.export_array_impl(&tmp).unwrap();
        let mut f = File::open(&tmp).unwrap();
        let mut buf = [0u8; 6];
        f.read_exact(&mut buf).unwrap();
        // First record: length 0x0006, type 0x00, data type 0x02, version 0x0258
        assert_eq!(buf, [0x00, 0x06, 0x00, 0x02, 0x02, 0x58]);
    }

    #[test]
    fn step_exporter_creates_iso10303_file() {
        let tmp = std::env::temp_dir().join("shbt_test.step");
        let model = StepSolidModel::new();
        model
            .export_waveguide_impl(&tmp, 350e-6, 5e-6, 1.5e-6)
            .unwrap();
        let mut f = File::open(&tmp).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        assert!(s.starts_with("ISO-10303-21;"));
        assert!(s.contains("MANIFOLD_SOLID_BREP"));
        assert!(s.contains("CLOSED_SHELL"));
        assert!(s.contains("ADVANCED_FACE"));
    }

    #[test]
    fn gds_real8_round_trip_one() {
        let bytes = gds_real8(1.0);
        // Decode with the same fractional convention.
        let exp = (bytes[0] & 0x7F) as i32 - 64;
        let mut mantissa: u64 = 0;
        for i in 1..8 {
            mantissa = (mantissa << 8) | bytes[i] as u64;
        }
        let value = (mantissa as f64) * (16.0_f64).powi(exp - 14);
        assert!((value - 1.0).abs() < 1e-12);
    }

    #[test]
    fn gdsii_drc_passes_for_shbt_features() {
        let exporter = GdsiiMaskExporter::new();
        let (ok, violations) = exporter.validate_drc_impl();
        assert!(ok, "DRC failed: {:?}", violations);
        assert!(violations.is_empty());
    }
}
