mod load_ecs_info {
    use toybox::*;

    error_chain! {}

    #[system]
    struct TestSystem {}

    impl<'s> System<'s> for TestSystem {
        type SystemData = ();

        fn run(&mut self, _system_data: Self::SystemData) {}
    }

    #[test]
    fn load_ecs_info() -> Result<()> {
        let mut plugin_manager = PluginManager::default();
        plugin_manager.add_plugin("script_ts");
        plugin_manager.add_plugin("example_pong");

        for system in inventory::iter::<SystemInfo> {
            println!(
                "lod_ecs_info system. address: {}, type_id: {:?}, name: {}",
                system as *const _ as usize,
                system.system_type_id(),
                system.name()
            );
        }
        Ok(())
    }
}
