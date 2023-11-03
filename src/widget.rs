use anyhow::Result;
use fluvio_stream_model::{store::{k8::{K8MetaItem, K8ConvertError, default_convert_from_k8, K8ExtendedSpec}, MetadataStoreObject}, k8_types::{core::secret::SecretSpec, K8Obj, Crd, CrdNames, DefaultHeader}, core::{Spec, Status}};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WidgetSpec {
    pub disabled: bool
}

impl Spec for WidgetSpec {
    const LABEL: &'static str = "Widget";
    type IndexKey = String;
    type Status = WidgetStatus;
    type Owner = Self;
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WidgetStatus {
    pub phase: WidgetPhase,
}

impl std::fmt::Display for WidgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "WidgetStatus")
    }
}

impl Status for WidgetStatus {}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum WidgetPhase {
    Init,
    Disabled,
    Active,
}

impl Default for WidgetPhase {
    fn default() -> Self {
        WidgetPhase::Init
    }
}

pub const GROUP: &str = "example.com";
pub const V1: &str = "v1";

const WIDGET_CRD: Crd = Crd {
    group: GROUP,
    version: V1,
    names: CrdNames {
        kind: "Widget",
        plural: "widgets",
        singular: "widget",
    },
};

impl fluvio_stream_model::k8_types::Spec for WidgetSpec {
    type Status = WidgetStatus;
    type Header = DefaultHeader;

    fn metadata() -> &'static Crd {
        &WIDGET_CRD
    }
}

impl fluvio_stream_model::k8_types::Status for WidgetStatus {}

impl K8ExtendedSpec for WidgetSpec {
    type K8Spec = Self;

    fn convert_from_k8(
        k8_obj: K8Obj<Self::K8Spec>,
        multi_namespace_context: bool,
    ) -> Result<MetadataStoreObject<Self, K8MetaItem>, K8ConvertError<Self::K8Spec>> {
        default_convert_from_k8(k8_obj, multi_namespace_context)
    }

    fn convert_status_from_k8(status: Self::Status) -> Self::Status {
        status
    }

    fn into_k8(self) -> Self::K8Spec {
        self
    }
}