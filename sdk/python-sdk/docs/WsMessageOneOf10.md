# WsMessageOneOf10

A result for a DNS resolution requests

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**attack_uuid** | **str** | The corresponding id of the attack | 
**source** | **str** | The source address that was queried | 
**destination** | **str** | The destination address that was returned | 
**type** | **str** |  | 

## Example

```python
from kraken_sdk.models.ws_message_one_of10 import WsMessageOneOf10

# TODO update the JSON string below
json = "{}"
# create an instance of WsMessageOneOf10 from a JSON string
ws_message_one_of10_instance = WsMessageOneOf10.from_json(json)
# print the JSON string representation of the object
print WsMessageOneOf10.to_json()

# convert the object into a dict
ws_message_one_of10_dict = ws_message_one_of10_instance.to_dict()
# create an instance of WsMessageOneOf10 from a dict
ws_message_one_of10_form_dict = ws_message_one_of10.from_dict(ws_message_one_of10_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


