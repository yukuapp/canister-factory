use candid::{CandidType, Principal, Deserialize, Encode, Nat, candid_method, export_service};
use ic_cdk_macros::*;
use icrc_ledger_types::icrc1::account::Account;
use ic_cdk::api::management_canister::main::{
    CreateCanisterArgument, CanisterIdRecord, CanisterSettings, CanisterInstallMode, InstallCodeArgument,
};
use ic_cdk::api::call::{CallResult, call};
use serde::Serialize;
#[derive(CandidType, Debug, Clone, Deserialize, Serialize)]
pub struct MintArgs {
    pub id: u128,
    pub name: String,
    pub description: Option<String>,
    pub image: String,
    pub to: Account,
    pub canister_name: String,
    pub canister_id: String,
}


#[derive(CandidType, Debug, Clone, Deserialize, Serialize)]
pub struct Args {
    pub id: u128,
    pub name: String,
    pub description: Option<String>,
    pub to: Account,
    pub image: Option<Vec<u8>>,
}

#[derive(CandidType, Debug, Clone, Deserialize, Serialize)]
pub struct InitArg{
    pub name: String,
    pub symbol: String,
    pub tx_window: u16,
    pub permitted_drift: u16,
    pub minting_authority: Option<Principal>,
    pub royalties: Option<u16>,
    pub royalties_recipient: Option<Account>,
    pub description: Option<String>,
    pub image: Option<Vec<u8>>,
    pub supply_cap: Option<u128>,
    pub wasm_name: String
}

#[derive(CandidType, Deserialize, Clone)]
pub struct CreateArg{
    pub name: String,
    pub symbol: String,
    pub royalties: Option<u16>,
    pub royalties_recipient: Option<Account>,
    pub description: Option<String>,
    pub image: Option<Vec<u8>>,
    pub supply_cap: Option<u128>,
    pub wasm_name: String
}

#[derive(CandidType, Deserialize, Clone, Debug, Serialize)]
pub struct ICRC7Response(u128);
#[derive(CandidType, Deserialize, Clone, Debug, Serialize)]
pub struct ICRC7Err(String);
#[derive(CandidType, Deserialize, Clone, Debug, Serialize)]

pub enum  MintResponse {
    ok(ICRC7Response),
    other(String),
    err(ICRC7Err),
}

impl From<(Principal, CreateArg)> for InitArg{
    fn from((minting_authority, arg): (Principal, CreateArg)) -> Self {
        InitArg { 
            name: arg.name, 
            symbol: arg.symbol, 
            tx_window: 1,
            permitted_drift: 1,
            minting_authority: Some(minting_authority), 
            royalties: arg.royalties, 
            royalties_recipient: arg.royalties_recipient, 
            description: arg.description, 
            image: arg.image, 
            supply_cap: arg.supply_cap ,
            wasm_name: arg.wasm_name
        }
    }
}


pub async fn get_an_address(caller: &Principal) -> Principal{
    let canister_setting = CanisterSettings{
        controllers: Some(vec![caller.clone(), ic_cdk::id()]),
        compute_allocation: Some(Nat::from(0_u64)),
        memory_allocation: Some(Nat::from(0_u64)),
        freezing_threshold: Some(Nat::from(0_u64)),
    };
    let args = CreateCanisterArgument{
        settings: Some(canister_setting)
    };
    let (canister_id,): (CanisterIdRecord,) = match ic_cdk::api::call::call_with_payment(Principal::management_canister(), "create_canister", (args,), 200_000_000_000).await
    {
        Ok(x) => x,
        Err((_, _)) => (CanisterIdRecord{
            canister_id: candid::Principal::anonymous()
        },),
    };
    canister_id.canister_id
}

pub async fn install_wasm(wasm: Vec<u8>, canister_id: Principal, args: Vec<u8>,) -> bool{
    let install_config = InstallCodeArgument{
        mode: CanisterInstallMode::Install,
        wasm_module: wasm,
        canister_id,
        arg: args
    };
    match ic_cdk::api::call::call(Principal::management_canister(), "install_code", (install_config,)).await
    {
        Ok(x) => x,
        Err((rejection_code, msg)) =>{
            return false
        }
    }
    true
}
const ICRC7_WASM: &[u8] = std::include_bytes!("/Users/yumeng/icrc7/target/wasm32-unknown-unknown/release/icrc7.wasm.gz");
const DIP721_WASM: &[u8] = std::include_bytes!("/Users/yumeng/icrc7/target/wasm32-unknown-unknown/release/ext_based_dip721_lib.wasm.gz");

fn choose_wasm(wasm_name: &str) -> Vec<u8>{
    match wasm_name {
        "icrc7" => return ICRC7_WASM.to_vec(),
        "dip721" => return DIP721_WASM.to_vec(),
        _ => return vec![],
    }
}

const icrc7: &str = "6222m-vqaaa-aaaah-adova-cai";
const dip721: &str = "6222m-vqaaa-aaaah-adova-cai";
async fn mint_internal(args: &MintArgs) -> MintResponse {
    let canister_id = &args.canister_id;
    let p_canister_id = Principal::from_text(canister_id).unwrap();
    let canister_name = args.canister_name.as_str();
    let image = serde_json::to_vec(&args.image).unwrap();
    let mint_args = Args {
        id: args.id,
        name: args.name.clone(),
        description: args.description.clone(),
        to: args.to,
        image: Some(image),
    };
    let result = match canister_name {
        "icrc7" => {
            
            let result: CallResult<(u128,)> = ic_cdk::api::call::call(p_canister_id, "icrc7_mint", (mint_args, )).await;
               
            
            let mint_res = match result
             {
                Ok(res) => MintResponse::ok(ICRC7Response(res.0)),
                // Ok((res, )) => match res {
                //     Ok(r) => 
                //         return MintResponse::ok(ICRC7Response(r)),
                //     Err(e) => return MintResponse::err(ICRC7Err(e)),
                // }
                         
                Err((_, e)) => MintResponse::err(ICRC7Err(e)),
            };
            return mint_res;
        },
        // "dip721"=> {
        //     let result = match ic_cdk::api::call::call(p_canister_id, "dip721_mint", (mint_args, )).await {
        //         Ok(x) => x,
        //         Err(_) => (),
        //     };
        //     return result.to_string();
        // },
        _ => MintResponse::other("invalid canister id".to_string()),
    };
    result
}


#[update]
#[candid_method(update)]
pub async fn create_collection(
    create_arg: CreateArg
) -> Principal{
    let caller = ic_cdk::caller();
    let init_arg = InitArg::from((caller, create_arg.clone()));
    let address = get_an_address(&caller).await;
    if address == Principal::anonymous(){
        ic_cdk::trap("Failed to get an address")
    }
    let wasm_name = create_arg.wasm_name;
    let arg = Encode!(&init_arg).unwrap();
    let wasm = choose_wasm(&wasm_name);
    match install_wasm(wasm, address, arg).await{
        true => address,
        false => ic_cdk::trap("Failed to install code")
    }
}

#[update]
#[candid_method(update)]
pub async fn mint_proxy(
    args: MintArgs,
) -> MintResponse {
    let response =  mint_internal(&args).await;
    response
    
}

// #[update]
// #[candid_method(update)]
// pub async fn metadata_proxy(
//     args: MetaDataArgs,
// ) -> Me

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_candid() {
        use std::env;
        use std::fs::write;
        use std::path::PathBuf;

        let dir = PathBuf::from(env::current_dir().unwrap());
        write(dir.join("service.did"), export_candid()).expect("Write failed.");
    }
}

