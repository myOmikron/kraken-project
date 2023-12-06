# FinishRegisterRequest

The request to finish the registration of a security key

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **str** |  | 

## Example

```python
from kraken_sdk.models.finish_register_request import FinishRegisterRequest

# TODO update the JSON string below
json = "{}"
# create an instance of FinishRegisterRequest from a JSON string
finish_register_request_instance = FinishRegisterRequest.from_json(json)
# print the JSON string representation of the object
print FinishRegisterRequest.to_json()

# convert the object into a dict
finish_register_request_dict = finish_register_request_instance.to_dict()
# create an instance of FinishRegisterRequest from a dict
finish_register_request_form_dict = finish_register_request.from_dict(finish_register_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


