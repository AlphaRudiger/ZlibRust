use std::{fs::File, arch::asm};

#[allow(unused_imports)]

pub mod bitreader;
pub mod zlib;
pub mod zlib_errors;
pub mod inflate;
pub mod adler32;


// fn run() {
//     println!("Hello, world!");
//     App::new()
//         .add_plugins(DefaultPlugins)
//         .add_systems(Startup, setup)
//         .add_systems(PreUpdate, bevy::window::close_on_esc)
//         .run();
// }

// fn setup(mut commands: Commands, assets: ResMut<AssetServer>) {
//     commands.spawn(Camera2dBundle::default());
//     let data = vec![255, 2, 3, 255];
//     let mut image = Image::new_fill(Extent3d { width: 1, height: 1, depth_or_array_layers: 1}, TextureDimension::D2, &data, TextureFormat::Rgba8Unorm);
    
//     let handle = assets.add(image);
    
//     commands.spawn(SpriteBundle {
//         texture: handle, 
//         ..Default::default()
//     });
// }
