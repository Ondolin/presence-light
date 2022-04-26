use crate::{ws, state::{CHANNEL, State}};
use warp::{Reply, http};

pub async fn post_current(state: warp::hyper::body::Bytes) -> Result<impl warp::Reply, warp::Rejection> {

    let state: Result<State, ()> = state.try_into();
   
    if state.is_err() {
        return Err(warp::reject::reject());
    }

    let state = state.unwrap();

    let channel = CHANNEL.clone();

    channel.send(state).expect("COULD NOT SEND TO CHANNEL");

    let output_string = format!("Changed state to: {}", state.to_str());

    Ok(warp::reply::with_status(
            output_string,
            http::StatusCode::CREATED,
    ))
}
