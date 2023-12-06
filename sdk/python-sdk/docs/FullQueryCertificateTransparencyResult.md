# FullQueryCertificateTransparencyResult

A simple representation of a query certificate transparency result

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** | The primary key | 
**attack** | **str** | The attack which produced this result | 
**created_at** | **datetime** | The point in time, this result was produced | 
**issuer_name** | **str** | The name of the issuer | 
**common_name** | **str** | The common name of the certificate | 
**value_names** | **List[str]** | The values of the certificate | 
**not_before** | **datetime** | The start date of the certificate | [optional] 
**not_after** | **datetime** | The end date of the certificate | [optional] 
**serial_number** | **str** | The serial number of the certificate | 

## Example

```python
from kraken_sdk.models.full_query_certificate_transparency_result import FullQueryCertificateTransparencyResult

# TODO update the JSON string below
json = "{}"
# create an instance of FullQueryCertificateTransparencyResult from a JSON string
full_query_certificate_transparency_result_instance = FullQueryCertificateTransparencyResult.from_json(json)
# print the JSON string representation of the object
print FullQueryCertificateTransparencyResult.to_json()

# convert the object into a dict
full_query_certificate_transparency_result_dict = full_query_certificate_transparency_result_instance.to_dict()
# create an instance of FullQueryCertificateTransparencyResult from a dict
full_query_certificate_transparency_result_form_dict = full_query_certificate_transparency_result.from_dict(full_query_certificate_transparency_result_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


