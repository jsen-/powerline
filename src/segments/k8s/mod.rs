mod kubeconfig;
mod kubeconfig_error;

use crate::{Color, ColoredStream, Segment};
use kubeconfig_error::ConfigError;
use std::{env, io::Write, path::Path};

pub struct K8s {
    server_name: Option<kubeconfig::Result<String>>,
}

fn servername_from_kubeconfig<P: AsRef<Path>>(path: P) -> kubeconfig::Result<String> {
    let config = kubeconfig::Kubeconfig::read_from(path)?;

    let context_name = if let Some(name) = &config.current_context {
        name
    } else {
        return Err(ConfigError::CurrentContextNotSet.into());
    };
    let current_context = config
        .contexts
        .iter()
        .find(|named_context| &named_context.name == context_name)
        .map(|named_context| &named_context.context)
        .ok_or_else(|| ConfigError::LoadContext {
            context_name: context_name.clone(),
        })?;

    let cluster_name = &current_context.cluster;
    let cluster = config
        .clusters
        .iter()
        .find(|named_cluster| &named_cluster.name == cluster_name)
        .map(|named_cluster| &named_cluster.cluster)
        .ok_or_else(|| ConfigError::LoadClusterOfContext {
            cluster_name: cluster_name.clone(),
        })?;

    // let user_name = &current_context.user;
    // let user = config
    //     .auth_infos
    //     .iter()
    //     .find(|named_user| &named_user.name == user_name)
    //     .map(|named_user| &named_user.auth_info)
    //     .ok_or_else(|| ConfigError::FindUser {
    //         user_name: user_name.clone(),
    //     })?;

    // Ok(ConfigLoader {
    //     current_context: current_context.clone(),
    //     cluster: cluster.clone(),
    //     user: user.clone(),
    // })

    Ok(cluster.server.clone())
}

impl K8s {
    pub fn new() -> Self {
        if let Some(kubeconfig_path) = env::var_os("KUBECONFIG") {
            Self {
                server_name: Some(servername_from_kubeconfig(kubeconfig_path)),
            }
        } else {
            dirs::home_dir()
                .and_then(|home| {
                    servername_from_kubeconfig(home.join(".kube").join("config"))
                        .ok()
                        .map(|server_name| Self {
                            server_name: Some(Ok(server_name)),
                        })
                })
                .unwrap_or(Self { server_name: None })
        }
    }
}

impl Segment for K8s {
    fn write(&mut self, w: &mut ColoredStream) -> std::io::Result<()> {
        if let Some(ref server_name) = self.server_name {
            match server_name {
                Ok(server_name) => {
                    w.set_bg(Color::from_rgb(10, 10, 200))?;
                    w.set_fg(Color::from_rgb(230, 230, 230))?;
                    write!(w, " ☸  {} ", server_name)?;
                }
                Err(_) => {
                    w.set_bg(Color::from_rgb(255, 0, 0))?;
                    w.set_fg(Color::from_rgb(230, 230, 230))?;
                    write!(w, " ☠ ")?;
                }
            }
        }
        Ok(())
    }
}
