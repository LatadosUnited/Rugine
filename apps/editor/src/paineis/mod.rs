//! Os paineis do editor.
//!
//! Um modulo por painel, cada um com uma funcao `desenhar`. Eles nao guardam
//! estado proprio: tudo vive no `Editor`, para que a interface seja uma funcao
//! do estado e nao uma copia dele que pode divergir.

pub mod bordas;
pub mod cena;
pub mod hierarquia;
pub mod inspector;
pub mod menu;
pub mod projeto;
pub mod viewport;
