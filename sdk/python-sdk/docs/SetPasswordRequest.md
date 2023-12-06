# SetPasswordRequest

The request to set a new password for a user

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**current_password** | **str** |  | 
**new_password** | **str** |  | 

## Example

```python
from kraken_sdk.models.set_password_request import SetPasswordRequest

# TODO update the JSON string below
json = "{}"
# create an instance of SetPasswordRequest from a JSON string
set_password_request_instance = SetPasswordRequest.from_json(json)
# print the JSON string representation of the object
print SetPasswordRequest.to_json()

# convert the object into a dict
set_password_request_dict = set_password_request_instance.to_dict()
# create an instance of SetPasswordRequest from a dict
set_password_request_form_dict = set_password_request.from_dict(set_password_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


