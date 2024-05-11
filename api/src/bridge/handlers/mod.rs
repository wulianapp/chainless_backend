pub mod bind_eth_addr;
pub mod gen_bind_eth_addr_sig;
pub mod gen_deposit_sig;
pub mod get_binded_eth_addr;
pub mod list_deposit_order;
pub mod list_withdraw_order;
pub mod pre_withdraw;

fn paginate_vec<T: Sized + Clone>(input: Vec<T>, page_size: usize, page_number: usize) -> Vec<T> {
    let skip_count = page_size * (page_number - 1);
    let start_index = skip_count;

    if start_index >= input.len() {
        return Vec::<T>::new();
    }
    let mut end_index = skip_count + page_size;
    if end_index > input.len() {
        end_index = input.len();
    }

    input[start_index..end_index].to_vec()
}
