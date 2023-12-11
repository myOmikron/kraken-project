# WsMessageOneOf7

A result for hosts alive check

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**attack_uuid** | **str** | The corresponding id of the attack | 
**host** | **str** | A host which could be reached | 
**type** | **str** |  | 

## Example

```python
from kraken_sdk.models.ws_message_one_of7 import WsMessageOneOf7

# TODO update the JSON string below
json = "{}"
# create an instance of WsMessageOneOf7 from a JSON string
ws_message_one_of7_instance = WsMessageOneOf7.from_json(json)
# print the JSON string representation of the object
print WsMessageOneOf7.to_json()

# convert the object into a dict
ws_message_one_of7_dict = ws_message_one_of7_instance.to_dict()
# create an instance of WsMessageOneOf7 from a dict
ws_message_one_of7_form_dict = ws_message_one_of7.from_dict(ws_message_one_of7_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


