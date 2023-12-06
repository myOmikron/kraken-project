# Query

A query for dehashed

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**email** | [**SearchType**](SearchType.md) |  | 
**ip_address** | [**SearchType**](SearchType.md) |  | 
**username** | [**SearchType**](SearchType.md) |  | 
**password** | [**SearchType**](SearchType.md) |  | 
**hashed_password** | [**SearchType**](SearchType.md) |  | 
**name** | [**SearchType**](SearchType.md) |  | 
**domain** | [**SearchType**](SearchType.md) |  | 
**vin** | [**SearchType**](SearchType.md) |  | 
**phone** | [**SearchType**](SearchType.md) |  | 
**address** | [**SearchType**](SearchType.md) |  | 

## Example

```python
from kraken_sdk.models.query import Query

# TODO update the JSON string below
json = "{}"
# create an instance of Query from a JSON string
query_instance = Query.from_json(json)
# print the JSON string representation of the object
print Query.to_json()

# convert the object into a dict
query_dict = query_instance.to_dict()
# create an instance of Query from a dict
query_form_dict = query.from_dict(query_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


