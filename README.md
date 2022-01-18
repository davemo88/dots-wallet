# dots-wallet

This is a simple wallet service. To run with `docker-compose`, simply do  
`$ docker-compose up`  
The service will be at `localhost:5000`.

## Endpoints
### Create Wallet
`POST /wallet`. The request body may be one of two versions: 
* `{"V1Body": wallet_id}`
* `{"V2Body": [wallet_id, wallet_name] }`  

where `wallet_id` is a `u32` and `wallet_name` is a `String`. Examples with `curl`:  
```bash
$ curl -d '{"V1Body":65}' -H 'Content-Type: application/json' localhost:5000/wallet
{"status":"success","data":65,"message":null}

# create another wallet with the same id
$ curl -d '{"V1Body":65}' -H 'Content-Type: application/json' localhost:5000/wallet
{"status":"error","data":null,"message":"wallet already exists"}

# other request body version
$ curl -d '{"V2Body":[654,"Sally"]}' -H 'Content-Type: application/json' localhost:5000/wallet
{"status":"success","data":654,"message":null}
```
### Add Item to Wallet
`POST /wallet/:wallet_id`. Similary to above, the request body may be one of two versions:
* `{"V1Body": item_id}`
* `{"V2Body": [item_id, item_name] }`

where `item_id` is a `u32` and `item_name` is a `String`. Examples with `curl`:
```bash
$ curl -d '{"V1Body":77}' -H 'Content-Type: application/json' localhost:5000/wallet/65
{"status":"success","data":[65,77],"message":null}

# add the same item again
$ curl -d '{"V1Body":77}' -H 'Content-Type: application/json' localhost:5000/wallet/65  
{"status":"error","data":null,"message":"item already in wallet"}

# add the item to a non-existant wallet
$ curl -d '{"V1Body":77}' -H 'Content-Type: application/json' localhost:5000/wallet/64
{"status":"error","data":null,"message":"no such wallet"}

# other request body version
$ curl -d '{"V2Body":[123,"An Item"]}' -H 'Content-Type: application/json' localhost:5000/wallet/654
{"status":"success","data":[654,123],"message":null}
```
### Retrieve Item from Wallet
`GET /wallet/:wallet_id/item/:item_id`. Examples with `curl`:
```bash
$ curl localhost:5000/wallet/65/item/77
{"status":"success","data":77,"message":null}

# retrieve a non-existant item
$ curl localhost:5000/wallet/65/item/78
{"status":"error","data":null,"message":"no such item"}

$ curl localhost:5000/wallet/654/item/123
{"status":"success","data":123,"message":null}
```

