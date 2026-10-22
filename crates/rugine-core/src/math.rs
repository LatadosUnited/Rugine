//! Algebra linear da engine.
//!
//! Reexportamos a `glam` em vez de embrulhar: ela ja e SIMD-aware e bem
//! testada, e um wrapper proprio so adicionaria conversoes em todo call site.
//! O alias existe para que o resto da engine importe `rugine_core::math`,
//! deixando uma troca futura confinada a este arquivo.

pub use glam::*;

/// Convencoes de camera da Rugine.
///
/// A `glam` nao escolhe por nos: ela oferece combinacoes de handedness do mundo
/// (`rh`/`lh`) e de NDC da API grafica (`opengl`/`directx`/`vulkan`). Usar a
/// combinacao errada nao da erro de compilacao — da um depth buffer invertido
/// ou uma cena espelhada, que sao bugs caros de diagnosticar depois.
///
/// A Rugine fixa as convencoes aqui, uma vez:
///
/// - **Mundo right-handed, Y para cima**, como Blender e glTF. Importar asset
///   nao deve exigir espelhar nada.
/// - **NDC do `directx`**, que e o do wgpu/WebGPU: profundidade em `0.0..1.0`
///   com Y para cima. O nome do modulo e historico; nao implica usar DX12.
pub mod camera {
    use glam::{Mat4, Vec3};

    /// Projecao em perspectiva para o espaco de clip do wgpu.
    ///
    /// `vertical_fov` em radianos.
    pub fn perspective(vertical_fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Mat4 {
        glam::camera::rh::proj::directx::perspective(vertical_fov, aspect_ratio, near, far)
    }

    /// Projecao em perspectiva com plano distante no infinito e profundidade
    /// invertida (`near` mapeia para 1.0).
    ///
    /// E a escolha certa para cenas grandes: concentra a precisao do float
    /// perto da camera, onde o z-fighting aparece.
    pub fn perspective_infinite_reverse(
        vertical_fov: f32,
        aspect_ratio: f32,
        near: f32,
    ) -> Mat4 {
        glam::camera::rh::proj::directx::perspective_infinite_reverse(
            vertical_fov,
            aspect_ratio,
            near,
        )
    }

    /// Projecao ortografica para o espaco de clip do wgpu.
    pub fn orthographic(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Mat4 {
        glam::camera::rh::proj::directx::orthographic(left, right, bottom, top, near, far)
    }

    /// Matriz de view olhando de `eye` para `target`.
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
        glam::camera::rh::view::look_at_mat4(eye, target, up)
    }

    /// Matriz de view olhando de `eye` na direcao `dir`.
    pub fn look_to(eye: Vec3, dir: Vec3, up: Vec3) -> Mat4 {
        glam::camera::rh::view::look_to_mat4(eye, dir, up)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Guarda a convencao de NDC: se alguem trocar o modulo de projecao, este
    /// teste quebra antes de a cena ficar errada na tela.
    #[test]
    fn perspective_mapeia_profundidade_para_zero_um() {
        let proj = camera::perspective(std::f32::consts::FRAC_PI_2, 1.0, 0.1, 100.0);

        // Em espaco de view right-handed, a camera olha para -Z.
        let no_near = proj * Vec4::new(0.0, 0.0, -0.1, 1.0);
        let no_far = proj * Vec4::new(0.0, 0.0, -100.0, 1.0);

        let z_near = no_near.z / no_near.w;
        let z_far = no_far.z / no_far.w;

        assert!(z_near.abs() < 1e-4, "near deveria mapear para 0.0, deu {z_near}");
        assert!((z_far - 1.0).abs() < 1e-4, "far deveria mapear para 1.0, deu {z_far}");
    }

    #[test]
    fn look_at_coloca_o_alvo_na_frente() {
        let view = camera::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        let alvo = view * Vec4::new(0.0, 0.0, 0.0, 1.0);

        // Em view space right-handed, o que esta na frente tem Z negativo.
        assert!(alvo.z < 0.0, "o alvo deveria estar na frente, Z deu {}", alvo.z);
    }
}
