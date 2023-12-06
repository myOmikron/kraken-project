# CertificateTransparencyEntry

Entry of certificate transparency results

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**serial_number** | **str** | The serial number of the certificate | 
**issuer_name** | **str** | The name of the issuer for the certificate | 
**common_name** | **str** | The common name of the certificate | 
**value_names** | **List[str]** | The value names of the certificate | 
**not_before** | **datetime** | The point in time after the certificate is valid | [optional] 
**not_after** | **datetime** | The point in time before the certificate is valid | [optional] 

## Example

```python
from kraken_sdk.models.certificate_transparency_entry import CertificateTransparencyEntry

# TODO update the JSON string below
json = "{}"
# create an instance of CertificateTransparencyEntry from a JSON string
certificate_transparency_entry_instance = CertificateTransparencyEntry.from_json(json)
# print the JSON string representation of the object
print CertificateTransparencyEntry.to_json()

# convert the object into a dict
certificate_transparency_entry_dict = certificate_transparency_entry_instance.to_dict()
# create an instance of CertificateTransparencyEntry from a dict
certificate_transparency_entry_form_dict = certificate_transparency_entry.from_dict(certificate_transparency_entry_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


