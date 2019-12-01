ONE=127.0.0.1:8081
TWO=127.0.0.1:8082
THREE=127.0.0.1:8083

start_service() {
    RUST_LOG=jsonrpc_ring=trace RUST_BACKTRACE=1 ADDRESS=$1 NEXT=$2 target/debug/jsonrpc-ring > $3 2>&1 &
}

cargo build

start_service $ONE $TWO one.log
start_service $TWO $THREE two.log
start_service $THREE $ONE three.log

sleep 3

curl -H "Content-Type: application/json" --data-binary '{"jsonrpc:"2.0", "id":1, "method:" "start_roll_call", "params": []}' http://127.0.0.1:8081
