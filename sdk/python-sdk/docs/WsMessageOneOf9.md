# WsMessageOneOf9

A result to a certificate transparency request

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**attack_uuid** | **str** | The corresponding id of the attack | 
**entries** | [**List[CertificateTransparencyEntry]**](CertificateTransparencyEntry.md) | The entries of the result | 
**type** | **str** |  | 

## Example

```python
from kraken_sdk.models.ws_message_one_of9 import WsMessageOneOf9

# TODO update the JSON string below
json = "{}"
# create an instance of WsMessageOneOf9 from a JSON string
ws_message_one_of9_instance = WsMessageOneOf9.from_json(json)
# print the JSON string representation of the object
print WsMessageOneOf9.to_json()

# convert the object into a dict
ws_message_one_of9_dict = ws_message_one_of9_instance.to_dict()
# create an instance of WsMessageOneOf9 from a dict
ws_message_one_of9_form_dict = ws_message_one_of9.from_dict(ws_message_one_of9_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


