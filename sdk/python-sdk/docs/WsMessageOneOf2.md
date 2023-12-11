# WsMessageOneOf2

A notification about a started attack

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**attack** | [**SimpleAttack**](SimpleAttack.md) |  | 
**workspace** | [**SimpleWorkspace**](SimpleWorkspace.md) |  | 
**type** | **str** |  | 

## Example

```python
from kraken_sdk.models.ws_message_one_of2 import WsMessageOneOf2

# TODO update the JSON string below
json = "{}"
# create an instance of WsMessageOneOf2 from a JSON string
ws_message_one_of2_instance = WsMessageOneOf2.from_json(json)
# print the JSON string representation of the object
print WsMessageOneOf2.to_json()

# convert the object into a dict
ws_message_one_of2_dict = ws_message_one_of2_instance.to_dict()
# create an instance of WsMessageOneOf2 from a dict
ws_message_one_of2_form_dict = ws_message_one_of2.from_dict(ws_message_one_of2_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


