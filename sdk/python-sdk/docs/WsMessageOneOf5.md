# WsMessageOneOf5

A result for a subdomain enumeration using bruteforce DNS requests

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**attack_uuid** | **str** | The corresponding id of the attack | 
**source** | **str** | The source address that was queried | 
**destination** | **str** | The destination address that was returned | 
**type** | **str** |  | 

## Example

```python
from kraken_sdk.models.ws_message_one_of5 import WsMessageOneOf5

# TODO update the JSON string below
json = "{}"
# create an instance of WsMessageOneOf5 from a JSON string
ws_message_one_of5_instance = WsMessageOneOf5.from_json(json)
# print the JSON string representation of the object
print WsMessageOneOf5.to_json()

# convert the object into a dict
ws_message_one_of5_dict = ws_message_one_of5_instance.to_dict()
# create an instance of WsMessageOneOf5 from a dict
ws_message_one_of5_form_dict = ws_message_one_of5.from_dict(ws_message_one_of5_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


