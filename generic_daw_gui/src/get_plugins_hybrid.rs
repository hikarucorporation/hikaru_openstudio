fn get_installed_plugins(config: &Config) -> Task<PluginDescriptor> {
        let (sender, receiver) = smol::channel::unbounded();
        let clap_paths = config.clap_paths.clone();
        let vst3_paths: Vec<std::sync::Arc<std::path::Path>> = config.vst3_paths.iter().map(|p| std::sync::Arc::from(p.as_path())).collect();

        let sender_clap = sender.clone();
        let sender_vst3 = sender.clone();

        Task::batch([
                // 1. Escaneo de CLAP nativos
                Task::future(unblock(move || {
                        generic_daw_core::clap_host::get_installed_plugins(
                                DEFAULT_CLAP_PATHS.iter().chain(&clap_paths),
                                |descriptor| _ = sender_clap.try_send(descriptor),
                        );
                }))
                .discard(),

                // 2. Escaneo de VST3 nativos con tu motor LGPL
                Task::future(unblock(move || {
                        vst_host::get_installed_plugins(&vst3_paths, |vst3_descriptor| {
                                // Mapeamos el descriptor a tu estructura compatible con la GUI
                                let descriptor = PluginDescriptor {
                                        name: vst3_descriptor.name,
                                        vendor: vst3_descriptor.vendor,
                                        category: vst3_descriptor.category,
                                        version: vst3_descriptor.version,
                                        // Guardamos la ruta del .so para poder instanciarlo después
                                        module_path: vst3_descriptor.module_path,
                                        class_id: vst3_descriptor.class_id,
                                };
                                _ = sender_vst3.try_send(descriptor);
                        });
                }))
                .discard(),

                Task::stream(receiver),
        ])
}
