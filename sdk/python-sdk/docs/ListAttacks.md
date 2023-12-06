# ListAttacks

A list of attacks

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**attacks** | [**List[SimpleAttack]**](SimpleAttack.md) | The list of the attacks | 

## Example

```python
from kraken_sdk.models.list_attacks import ListAttacks

# TODO update the JSON string below
json = "{}"
# create an instance of ListAttacks from a JSON string
list_attacks_instance = ListAttacks.from_json(json)
# print the JSON string representation of the object
print ListAttacks.to_json()

# convert the object into a dict
list_attacks_dict = list_attacks_instance.to_dict()
# create an instance of ListAttacks from a dict
list_attacks_form_dict = list_attacks.from_dict(list_attacks_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


