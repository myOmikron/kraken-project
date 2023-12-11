# WsMessageOneOf8

A result for a tcp scan

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**attack_uuid** | **str** | The corresponding id of the attack | 
**address** | **str** | The address of the result | 
**port** | **int** | The port of the result | 
**type** | **str** |  | 

## Example

```python
from kraken_sdk.models.ws_message_one_of8 import WsMessageOneOf8

# TODO update the JSON string below
json = "{}"
# create an instance of WsMessageOneOf8 from a JSON string
ws_message_one_of8_instance = WsMessageOneOf8.from_json(json)
# print the JSON string representation of the object
print WsMessageOneOf8.to_json()

# convert the object into a dict
ws_message_one_of8_dict = ws_message_one_of8_instance.to_dict()
# create an instance of WsMessageOneOf8 from a dict
ws_message_one_of8_form_dict = ws_message_one_of8.from_dict(ws_message_one_of8_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


