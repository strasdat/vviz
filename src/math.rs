//! Math helper functions.

/// Canonical rotation about x-axis.
///
/// Returns pure rotational pose (aka [nalgebra::Isometry3<T>]) with zero translation component.
pub fn rot_x<T: nalgebra::RealField>(x: T) -> nalgebra::Isometry3<T> {
    let zero_trans = nalgebra::Translation3 {
        vector: nalgebra::Vector3::zeros(),
    };
    let mut scaled_axis = nalgebra::Vector3::zeros();
    scaled_axis[0] = x;
    nalgebra::Isometry3::from_parts(
        zero_trans,
        nalgebra::UnitQuaternion::from_scaled_axis(scaled_axis),
    )
}

/// Canonical rotation about y-axis.
///
/// Returns pure rotational pose (aka [nalgebra::Isometry3<T>]) with zero translation component.
pub fn rot_y<T: nalgebra::RealField>(y: T) -> nalgebra::Isometry3<T> {
    let zero_trans = nalgebra::Translation3 {
        vector: nalgebra::Vector3::zeros(),
    };
    let mut scaled_axis = nalgebra::Vector3::zeros();
    scaled_axis[1] = y;
    nalgebra::Isometry3::from_parts(
        zero_trans,
        nalgebra::UnitQuaternion::from_scaled_axis(scaled_axis),
    )
}

/// Canonical rotation about z-axis.
///
/// Returns pure rotational pose (aka [nalgebra::Isometry3<T>]) with zero translation component.
pub fn rot_z<T: nalgebra::RealField>(z: T) -> nalgebra::Isometry3<T> {
    let zero_trans = nalgebra::Translation3 {
        vector: nalgebra::Vector3::zeros(),
    };
    let mut scaled_axis = nalgebra::Vector3::zeros();
    scaled_axis[2] = z;
    nalgebra::Isometry3::from_parts(
        zero_trans,
        nalgebra::UnitQuaternion::from_scaled_axis(scaled_axis),
    )
}
