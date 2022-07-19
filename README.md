### Transaction processor 

To run: `cargo run -- transactions.csv > accounts.csv`

### It is possible to process such transactions:
- Deposit: increase client's account balance
- Withdrawal: increase client's account balance
- Dispute: puts client's transaction on hold
- Resolve: puts client's transaction back to account balance, ignore if no such transaction
- Chargeback: reverses client's transaction, and freeze client's account

To enable debug put `RUST_LOG=debug` in `.env` file.
