#[macro_export]
macro_rules! cli_commands {
    (
        $( $variant:ident ($ty:ty) ),* $(,)?
    ) => {
        #[derive(clap::Subcommand, Debug, Clone)]
        pub enum Commands {
            $(
                #[command(
                    aliases = <$ty as $crate::cmds::CommandMetadata>::aliases(),
                    visible_aliases = <$ty as $crate::cmds::CommandMetadata>::visible_aliases(),
                    about = <$ty as $crate::cmds::CommandMetadata>::about(),
                    long_about = <$ty as $crate::cmds::CommandMetadata>::long_about(),
                    hide = <$ty as $crate::cmds::CommandMetadata>::hide(),
                )]
                $variant($ty),
            )*
        }

        impl Commands {
            pub fn execute(
                &self,
                dev: &mut aurora::Device,
                config: &$crate::config::AuroraConfig,
            ) -> anyhow::Result<()> {
                match self {
                    $(
                        Commands::$variant(inner) => {
                            $crate::cmds::CliCommand::run(inner, dev, config)
                        }
                    )*
                }
            }
        }
    };
}
