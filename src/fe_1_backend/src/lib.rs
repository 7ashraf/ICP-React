#[macro_use]
extern crate serde;
use candid::{Decode, Deserialize, Encode, Principal};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};
use std::collections::BTreeMap;

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct Proposal {
    id: u64,
    title: String,
    description: String,
    approve: u64,
    reject: u64,
    pass: u64,
    votes: Vec<u64>,
    created_at: u64,
    owner: Principal,
    voted: Vec<Principal>,
    has_ended: bool, 

}

type IdStore = BTreeMap<String, Principal>;
type ProposalStore = BTreeMap<u64, Proposal>;


impl Storable for Proposal {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}


thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static ID_STORE: RefCell<IdStore> = RefCell::default();


    // static PROPOSAL_STORAGE: RefCell<StableBTreeMap<u64, Proposal, Memory>> =
    //     RefCell::new(StableBTreeMap::init(
    //         MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    // ));
    static PROPOSAL_STORAGE: RefCell<ProposalStore> = RefCell::default();
}
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct ProposalPayload {
    title: String,
    description: String,
}

#[ic_cdk::update]
fn add_proposal(payload: ProposalPayload) -> Option<Proposal> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");
    
    let proposal = Proposal {
        id,
        title: payload.title,
        description: payload.description,
        votes: vec![],
        created_at: time(),
        owner: ic_cdk::caller(),
        approve: 0,
        reject: 0,
        pass: 0,
        voted: vec![],
        has_ended: false,

    };
  

    PROPOSAL_STORAGE.with(|proposal_storage| proposal_storage.borrow_mut().insert(id, proposal.clone()));
    Some(proposal)
}
#[derive(candid::CandidType, Deserialize, Serialize)]
enum VoteError {
    NotFound { msg: String },
    InvalidOption { msg: String },
    Unauthorized { msg: String },
}

#[ic_cdk::update]
fn cast_vote(proposal_id: u64, option_index: usize) -> Result<Proposal, VoteError> {
    PROPOSAL_STORAGE.with(|proposal_storage| {
        let mut proposal_storage = proposal_storage.borrow_mut();
        if let Some(mut proposal) = proposal_storage.get(&proposal_id).cloned() {
            if option_index == 0{
                proposal.approve += 1;
                proposal_storage.insert(proposal_id, proposal.clone());
                Ok(proposal)
            } else if option_index == 1{
                proposal.reject += 1;
                proposal_storage.insert(proposal_id, proposal.clone());
                Ok(proposal)
            } else if option_index == 2{
                proposal.pass += 1;
                proposal_storage.insert(proposal_id, proposal.clone());
                Ok(proposal)
            
            }
            else if option_index == 3{
                proposal.pass += 1;
                proposal_storage.insert(proposal_id, proposal.clone());
                Ok(proposal)
            }
            
            else {
                Err(VoteError::InvalidOption {
                    msg: format!("Invalid option index: {}", option_index),
                })
            }
        } else {
            Err(VoteError::NotFound {
                msg: format!("Proposal with id={} not found", proposal_id),
            })
        }
    })
}
#[ic_cdk::query]
fn get_proposal(proposal_id: u64) -> Result<Proposal, VoteError> {
    PROPOSAL_STORAGE.with(|proposal_storage| {
        match proposal_storage.borrow().get(&proposal_id) {
            Some(proposal) => Ok(proposal.clone()),
            None => Err(VoteError::NotFound {
                msg: format!("Proposal with id={} not found", proposal_id),
            }),
        }
    })
}

#[ic_cdk::query]
fn get_all_proposals() -> Vec<Proposal> {
    PROPOSAL_STORAGE.with(|proposal_storage| {
        proposal_storage
            .borrow()
            .iter()
            .map(|(_, proposal)| proposal.clone())
            .collect()
    })
}
#[derive(candid::CandidType, Serialize, Deserialize)]
struct EditProposalPayload {
    id: u64,
    title: Option<String>,
    description: Option<String>,
    options: Option<Vec<String>>,
}

#[ic_cdk::update]
fn edit_proposal(payload: EditProposalPayload) -> Result<Proposal, VoteError> {
    PROPOSAL_STORAGE.with(|proposal_storage| {
        let mut proposal_storage = proposal_storage.borrow_mut();
        if let Some(mut proposal) = proposal_storage.get(&payload.id).cloned() {
            if proposal.owner != ic_cdk::caller() {
                return Err(VoteError::Unauthorized {
                    msg: "Only the owner can edit the proposal".to_string(),
                });
            }

            if proposal.has_ended {
                return Err(VoteError::Unauthorized {
                    msg: "Cannot edit an ended proposal".to_string(),
                });
            }

            if let Some(title) = payload.title {
                proposal.title = title;
            }

            if let Some(description) = payload.description {
                proposal.description = description;
            }

            proposal.reject = 0;
            proposal.approve = 0;
            proposal.pass = 0;

            proposal_storage.insert(payload.id, proposal.clone());
            Ok(proposal)
        } else {
            Err(VoteError::NotFound {
                msg: format!("Proposal with id={} not found", payload.id),
            })
        }
    })
}
#[ic_cdk::update]
fn end_proposal(proposal_id: u64) -> Result<Proposal, VoteError> {
    PROPOSAL_STORAGE.with(|proposal_storage| {
        let mut proposal_storage = proposal_storage.borrow_mut();
        if let Some(mut proposal) = proposal_storage.get(&proposal_id).cloned() {
            if proposal.owner != ic_cdk::caller() {
                return Err(VoteError::Unauthorized {
                    msg: "Only the owner can end the proposal".to_string(),
                });
            }

            if proposal.has_ended {
                return Err(VoteError::Unauthorized {
                    msg: "Proposal is already ended".to_string(),
                });
            }

            proposal.has_ended = true;
            proposal_storage.insert(proposal_id, proposal.clone());
            Ok(proposal)
        } else {
            Err(VoteError::NotFound {
                msg: format!("Proposal with id={} not found", proposal_id),
            })
        }
    })
}
ic_cdk::export_candid!();
