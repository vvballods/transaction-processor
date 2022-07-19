### Transaction processor 

To run: `cargo run -- transactions.csv > accounts.csv`

Deposit: increase client's account balance
Withdrawal: increase client's account balance
Dispute: puts client's transaction on hold
Resolve: puts client's transaction back to account balance, ignore if no such transaction
Chargeback: reverses client's transaction, and freeze client's account