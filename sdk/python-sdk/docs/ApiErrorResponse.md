# ApiErrorResponse

Representation of an error response  `status_code` holds the error code, `message` a human readable description of the error

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**status_code** | [**ApiStatusCode**](ApiStatusCode.md) |  | 
**message** | **str** |  | 

## Example

```python
from kraken_sdk.models.api_error_response import ApiErrorResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ApiErrorResponse from a JSON string
api_error_response_instance = ApiErrorResponse.from_json(json)
# print the JSON string representation of the object
print ApiErrorResponse.to_json()

# convert the object into a dict
api_error_response_dict = api_error_response_instance.to_dict()
# create an instance of ApiErrorResponse from a dict
api_error_response_form_dict = api_error_response.from_dict(api_error_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


