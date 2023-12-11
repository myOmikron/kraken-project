# WsMessageOneOf11

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
from kraken_sdk.models.ws_message_one_of11 import WsMessageOneOf11

# TODO update the JSON string below
json = "{}"
# create an instance of WsMessageOneOf11 from a JSON string
ws_message_one_of11_instance = WsMessageOneOf11.from_json(json)
# print the JSON string representation of the object
print WsMessageOneOf11.to_json()

# convert the object into a dict
ws_message_one_of11_dict = ws_message_one_of11_instance.to_dict()
# create an instance of WsMessageOneOf11 from a dict
ws_message_one_of11_form_dict = ws_message_one_of11.from_dict(ws_message_one_of11_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


