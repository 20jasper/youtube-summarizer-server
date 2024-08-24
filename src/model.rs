//! Model layer with mock store layer for prototyping

#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use crate::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
pub struct Ticket {
	id: u64,
	title: String,
}

#[derive(Deserialize)]
pub struct ClientTicket {
	title: String,
}

pub struct Controller {
	ticket_store: Arc<Mutex<Vec<Option<Ticket>>>>,
}

impl Controller {
	#[allow(clippy::unused_async)]
	pub async fn new(ticket_store: Arc<Mutex<Vec<Option<Ticket>>>>) -> Result<Self> {
		Ok(Self { ticket_store })
	}
}

impl Controller {
	#[allow(clippy::unused_async)]
	pub async fn create_ticket(&self, ClientTicket { title }: ClientTicket) -> Result<Ticket> {
		let mut store = self.ticket_store.lock().unwrap();
		let id: u64 = store.len().try_into().unwrap();

		let ticket = Ticket { id, title };

		store.push(Some(ticket.clone()));

		Ok(ticket)
	}

	#[allow(clippy::unused_async)]
	pub async fn list_tickets(&self) -> Result<Vec<Ticket>> {
		let store = self.ticket_store.lock().unwrap();

		let tickets = store
			.iter()
			.filter_map(Option::as_ref)
			.cloned()
			.collect::<Vec<Ticket>>();

		Ok(tickets)
	}

	#[allow(clippy::unused_async)]
	pub async fn delete_ticket(&self, id: u64) -> Result<Ticket> {
		let mut store = self.ticket_store.lock().unwrap();

		let ticket = store
			.get_mut(usize::try_from(id).unwrap())
			.and_then(Option::take);

		ticket.ok_or(Error::TicketDeleteFailureIdNotFound { id })
	}
}
