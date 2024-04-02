use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use swaggapi::as_responses::AsResponses;
use swaggapi::internals::SchemaGenerator;
use swaggapi::re_exports::indexmap::IndexMap;
use swaggapi::re_exports::openapiv3::ReferenceOr;
use swaggapi::re_exports::openapiv3::Response;
use swaggapi::re_exports::openapiv3::Responses;
use swaggapi::re_exports::openapiv3::StatusCode;

use crate::chan::ws_manager::schema::WsClientMessage;
use crate::chan::ws_manager::schema::WsMessage;

/// Wrapper around [`HttpResponse`] with a custom [`AsResponses`] impl
pub struct WebsocketUpgrade(pub HttpResponse);

impl Responder for WebsocketUpgrade {
    type Body = <HttpResponse as Responder>::Body;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        <HttpResponse as Responder>::respond_to(self.0, req)
    }
}

impl AsResponses for WebsocketUpgrade {
    fn responses(gen: &mut SchemaGenerator) -> Responses {
        gen.generate::<WsMessage>();
        gen.generate::<WsClientMessage>();
        Responses {
            default: None,
            responses: IndexMap::from_iter([(
                StatusCode::Code(101),
                ReferenceOr::Item(Response {
                    description: "Upgrade to websocket".to_string(),
                    headers: Default::default(),
                    content: Default::default(),
                    links: Default::default(),
                    extensions: Default::default(),
                }),
            )]),
            extensions: Default::default(),
        }
    }
}
