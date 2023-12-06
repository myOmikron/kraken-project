# LeechConfig

The configuration of a leech

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**ca** | **str** | PEM encoded CA managed by kraken | 
**cert** | **str** | PEM encoded certificate | 
**key** | **str** | PEM encoded private key for the certificate | 
**sni** | **str** | The randomly generated fake domain for the kraken to be used for sni | 
**secret** | **str** |  | 

## Example

```python
from kraken_sdk.models.leech_config import LeechConfig

# TODO update the JSON string below
json = "{}"
# create an instance of LeechConfig from a JSON string
leech_config_instance = LeechConfig.from_json(json)
# print the JSON string representation of the object
print LeechConfig.to_json()

# convert the object into a dict
leech_config_dict = leech_config_instance.to_dict()
# create an instance of LeechConfig from a dict
leech_config_form_dict = leech_config.from_dict(leech_config_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


