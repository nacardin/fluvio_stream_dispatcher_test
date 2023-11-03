use std::{sync::Arc, time::Duration};

use anyhow::Result;
use fluvio_future::{timer::after, task::{spawn, JoinHandle}};
use fluvio_stream_dispatcher::{store::StoreContext, dispatcher::MetadataDispatcher};
use fluvio_stream_model::{store::{k8::{K8MetaItem, K8ConvertError, default_convert_from_k8, K8ExtendedSpec}, MetadataStoreObject}, k8_types::{core::secret::SecretSpec, K8Obj, Crd, CrdNames, DefaultHeader, ObjectMeta}, core::{Spec, Status}};
use k8_client::K8Client;

mod widget;

use widget::{WidgetSpec, WidgetStatus, WidgetPhase};

const K8_NAMESPACE: &str = "default";

#[async_std::main]
async fn main() -> Result<()> {


    let client = Arc::new(K8Client::try_default()?);
    
    let ctx: StoreContext<WidgetSpec, K8MetaItem> = StoreContext::new();

    MetadataDispatcher::<_, _, K8MetaItem>::start(
        K8_NAMESPACE.to_owned(),
        client,
        ctx.clone(),
    );

    let controller = WidgetController {
        widgets: ctx.clone()
    };

    controller.start();

    after(Duration::from_secs(5)).await;


    let spec = WidgetSpec {
        disabled: false
    };

    let widget_name = "w1";

    let mso = ctx.create_spec(widget_name.to_owned(), spec).await?;

    let owner_ref = mso.ctx.item().make_owner_reference::<WidgetSpec>();

    after(Duration::from_secs(5)).await;

    let widget_name = "w2";
    
    let spec = WidgetSpec {
        disabled: false
    };

    let widget_metadata = ObjectMeta {
        name: widget_name.to_owned(),
        namespace: K8_NAMESPACE.to_owned(),
        owner_references: vec![owner_ref],
        ..Default::default()
    };

    let widget = K8Obj {
        api_version: <WidgetSpec as fluvio_stream_model::k8_types::Spec>::api_version(),
        kind: <WidgetSpec as fluvio_stream_model::k8_types::Spec>::kind(),
        spec,
        metadata: widget_metadata,
        ..Default::default()
    };

    let widget_mso = default_convert_from_k8(widget, false).unwrap();

    let _mso = ctx.apply(widget_mso).await?;

    after(Duration::from_secs(5)).await;

    let spec = WidgetSpec {
        disabled: false
    };

    let widget_name = "w2";

    let _mso = ctx.create_spec(widget_name.to_owned(), spec).await?;

    after(Duration::from_secs(60)).await;
    Ok(())
}


pub struct WidgetController {
    pub widgets: StoreContext<WidgetSpec, K8MetaItem>,
}

impl WidgetController {
    pub fn start(mut self) -> JoinHandle<()> {
        spawn(self.dispatch_loop())
    }

    async fn dispatch_loop(mut self) {
        loop {
            after(Duration::from_secs(5)).await;
            self.inner_loop().await;
        }
    }

    async fn inner_loop(&self) {
        use tokio::select;

        let mut cluster_listener = self.widgets.change_listener();

        if !cluster_listener.has_change() {
            println!("no changes in clusters");
            return;
        }

        let changes = cluster_listener.sync_changes().await;

        if changes.is_empty() {
            println!("no changes in clusters");
            return;
        }

        let (updates, _deletes) = changes.parts();

        for update in updates {
            let _updated_mso = self.widgets.update_status(update.key, WidgetStatus {
                phase: match update.spec.disabled {
                    true => WidgetPhase::Disabled,
                    false => WidgetPhase::Active,
                }
            }).await;                
        }

        after(Duration::from_secs(5)).await;

        println!("waiting for events");

        select! {
            _ = cluster_listener.listen() => {
                println!("clusters change detected");
            }
        }
    }
}
