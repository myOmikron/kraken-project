# QueryCertificateTransparencyRequest

The settings to configure a certificate transparency request

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**target** | **str** | Domain to query certificates for | 
**include_expired** | **bool** | Should expired certificates be included as well | 
**max_retries** | **int** | The number of times the query should be retried if it failed. | 
**retry_interval** | **int** | The interval that should be waited between retries.  The interval is specified in milliseconds. | 
**workspace_uuid** | **str** | The workspace to execute the attack in | 

## Example

```python
from kraken_sdk.models.query_certificate_transparency_request import QueryCertificateTransparencyRequest

# TODO update the JSON string below
json = "{}"
# create an instance of QueryCertificateTransparencyRequest from a JSON string
query_certificate_transparency_request_instance = QueryCertificateTransparencyRequest.from_json(json)
# print the JSON string representation of the object
print QueryCertificateTransparencyRequest.to_json()

# convert the object into a dict
query_certificate_transparency_request_dict = query_certificate_transparency_request_instance.to_dict()
# create an instance of QueryCertificateTransparencyRequest from a dict
query_certificate_transparency_request_form_dict = query_certificate_transparency_request.from_dict(query_certificate_transparency_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


