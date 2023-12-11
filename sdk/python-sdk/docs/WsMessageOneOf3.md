# WsMessageOneOf3

A notification about a finished attack

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**attack** | [**SimpleAttack**](SimpleAttack.md) |  | 
**type** | **str** |  | 

## Example

```python
from kraken_sdk.models.ws_message_one_of3 import WsMessageOneOf3

# TODO update the JSON string below
json = "{}"
# create an instance of WsMessageOneOf3 from a JSON string
ws_message_one_of3_instance = WsMessageOneOf3.from_json(json)
# print the JSON string representation of the object
print WsMessageOneOf3.to_json()

# convert the object into a dict
ws_message_one_of3_dict = ws_message_one_of3_instance.to_dict()
# create an instance of WsMessageOneOf3 from a dict
ws_message_one_of3_form_dict = ws_message_one_of3.from_dict(ws_message_one_of3_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


