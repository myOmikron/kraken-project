# Color

Color value

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r** | **int** | Red value | 
**g** | **int** | Green value | 
**b** | **int** | Blue value | 
**a** | **int** | Alpha value | 

## Example

```python
from kraken_sdk.models.color import Color

# TODO update the JSON string below
json = "{}"
# create an instance of Color from a JSON string
color_instance = Color.from_json(json)
# print the JSON string representation of the object
print Color.to_json()

# convert the object into a dict
color_dict = color_instance.to_dict()
# create an instance of Color from a dict
color_form_dict = color.from_dict(color_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


