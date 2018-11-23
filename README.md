# 交易撮合系统简易DEMO


#### 运行程序


```
cd matched_trading
cargo run
```

#### 1. 查询交易对支持的列表


```
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "list", "id":123 }' 127.0.0.1:3030
```


#### 2. 创建交易对


```
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "create_order_book", "id":123, "params": {"code": "cet_eos"} }' 127.0.0.1:3030
```


#### 3. 删除交易对


```
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "remove_order_book", "id":123, "params": {"code": "cet_eos"} }' 127.0.0.1:3030
```


#### 4. 查询交易对买卖列表


```
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "order_book", "id":123, "params": {"code": "cet_eos"} }' 127.0.0.1:3030
```


#### 5. 下单买


```
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "submit_order", "id":123, "params": {"code": "cet_eos", "direction": "Ask", "id": 1, "price": 2.0, "volume": 1} }' 127.0.0.1:3030
```
如有匹配，则返回对应订单剩余的未成交量，例如：

```
{"jsonrpc":"2.0","result":[[1,"0.00000000"],[2,"0.90000000"]],"id":123}
```

#### 6. 下单卖


```
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "submit_order", "id":123, "params": {"code": "cet_eos", "direction": "Bid", "id": 2, "price": 2.0, "volume": 1} }' 127.0.0.1:3030
```

返回值同上
