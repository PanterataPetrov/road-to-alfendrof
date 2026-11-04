//! Sorting out Vulkan on the PCs where it doesn't start.
//!
//! The game draws with Vulkan where it can. On some Windows PCs, laptops with two graphics chips above all, an optional layer
//! the graphics drivers install (the switching between the chips, an overlay) makes Vulkan fail to list the graphics card
//! ("A return array was too small for the result"). The game then falls back to DirectX 12, which draws the sprites wrongly.
//! Turning those optional layers off fixes it, but they are only turned off on a PC that needs it: the game looks for
//! Vulkan adapters first, and only if there are none (with nothing set by the player) does it start itself again with the
//! layers off. If Vulkan still isn't there, that is remembered so the check isn't repeated on every start.

#[cfg(windows)]
pub fn sort_out() {
    use std::path::PathBuf;

    let args: Vec<String> = std::env::args().collect();
    // somebody asked for a graphics API or the layers already, so leave it alone
    if args.iter().any(|a| a == "--backend") || std::env::var_os("WGPU_BACKEND").is_some() {
        return;
    }
    let retry = std::env::var_os("T2F_VK_RETRY").is_some();
    if !retry && std::env::var_os("VK_LOADER_LAYERS_DISABLE").is_some() {
        return;
    }
    let marker: Option<PathBuf> = std::env::var_os("APPDATA").map(|d| PathBuf::from(d).join("Road to Alfendrof").join("no_vulkan"));
    if marker.as_ref().is_some_and(|m| m.exists()) {
        return;
    }
    if vulkan_adapters() > 0 {
        return;
    }
    if retry {
        // started again with the layers off, and there is still no Vulkan: this PC doesn't have it. Remember that.
        if let Some(m) = &marker {
            let _ = std::fs::create_dir_all(m.parent().unwrap_or(m));
            let _ = std::fs::write(m, "no Vulkan graphics card was found, with or without the optional layers\n");
        }
        return;
    }
    // no Vulkan: start the game again with the optional layers off, and see
    if let Ok(exe) = std::env::current_exe() {
        let started = std::process::Command::new(exe).args(&args[1..]).env("VK_LOADER_LAYERS_DISABLE", "~implicit~").env("T2F_VK_RETRY", "1").spawn();
        if started.is_ok() {
            std::process::exit(0);
        }
    }
}

/// How many Vulkan graphics cards the Vulkan loader lists.
#[cfg(windows)]
fn vulkan_adapters() -> usize {
    // (`T2F_VK_FAKE_FAIL=1` makes a PC pretend, until the layers are off, that Vulkan lists nothing, to try this out)
    if std::env::var_os("T2F_VK_FAKE_FAIL").is_some() && std::env::var_os("VK_LOADER_LAYERS_DISABLE").is_none() {
        return 0;
    }
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::VULKAN;
    let instance = wgpu::Instance::new(descriptor);
    bevy::tasks::block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN)).len()
}

#[cfg(not(windows))]
pub fn sort_out() {}
