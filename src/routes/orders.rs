use crate::domain::order_entry::OrderEntry;
use crate::matching::command::MatchingEngineCommand;
use crate::matching::state::AppState;
use crate::routes::models::order_modification::{OrderDeletion, OrderModification};
use actix_web::{HttpResponse, delete, patch, post, web};
use log::debug;

#[post("/orders")]
async fn add_orders(
    state: web::Data<AppState>,
    entries: web::Json<Vec<OrderEntry>>,
) -> HttpResponse {
    let order_entries: Vec<OrderEntry> = entries.into_inner();

    for o in order_entries {
        state
            .tx
            .send(MatchingEngineCommand::Create(o))
            .await
            .expect("Matching engine is not matching!");
    }

    HttpResponse::Ok().finish()
}

#[delete("/orders")]
async fn remove_orders(
    state: web::Data<AppState>,
    orders: web::Json<Vec<OrderDeletion>>,
) -> HttpResponse {
    for o in orders.0 {
        if let Err(e) = state
            .tx
            .send(MatchingEngineCommand::Delete(o.id, o.revision))
            .await
        {
            debug!("Failed to send delete for order {:?}: {:?}", o.id, e);
            continue;
        }
    }

    HttpResponse::Ok().finish()
}

#[patch("/orders")]
async fn update_orders(
    state: web::Data<AppState>,
    orders: web::Json<Vec<OrderModification>>,
) -> HttpResponse {
    for o in orders.0 {
        if let Err(e) = state
            .tx
            .send(MatchingEngineCommand::Modify(
                o.id,
                o.revision,
                o.new_price,
                o.new_quantity,
            ))
            .await
        {
            debug!("Failed to modify order {:?}: {:?}", o.id, e);
            continue;
        }
    }

    HttpResponse::Ok().finish()
}
