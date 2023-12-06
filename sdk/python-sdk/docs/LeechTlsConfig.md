# LeechTlsConfig

The tls related part of a leech's config

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**ca** | **str** | PEM encoded CA managed by kraken | 
**cert** | **str** | PEM encoded certificate | 
**key** | **str** | PEM encoded private key for the certificate | 
**sni** | **str** | The randomly generated fake domain for the kraken to be used for sni | 

## Example

```python
from kraken_sdk.models.leech_tls_config import LeechTlsConfig

# TODO update the JSON string below
json = "{}"
# create an instance of LeechTlsConfig from a JSON string
leech_tls_config_instance = LeechTlsConfig.from_json(json)
# print the JSON string representation of the object
print LeechTlsConfig.to_json()

# convert the object into a dict
leech_tls_config_dict = leech_tls_config_instance.to_dict()
# create an instance of LeechTlsConfig from a dict
leech_tls_config_form_dict = leech_tls_config.from_dict(leech_tls_config_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


