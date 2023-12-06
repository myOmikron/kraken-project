# WsMessageOneOf

An invalid message was received.  This message type is sent to the client.

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**type** | **str** |  | 

## Example

```python
from kraken_sdk.models.ws_message_one_of import WsMessageOneOf

# TODO update the JSON string below
json = "{}"
# create an instance of WsMessageOneOf from a JSON string
ws_message_one_of_instance = WsMessageOneOf.from_json(json)
# print the JSON string representation of the object
print WsMessageOneOf.to_json()

# convert the object into a dict
ws_message_one_of_dict = ws_message_one_of_instance.to_dict()
# create an instance of WsMessageOneOf from a dict
ws_message_one_of_form_dict = ws_message_one_of.from_dict(ws_message_one_of_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


