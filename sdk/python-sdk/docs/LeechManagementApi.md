# kraken_sdk.LeechManagementApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_leech**](LeechManagementApi.md#create_leech) | **POST** /api/v1/admin/leeches | Create a leech
[**delete_leech**](LeechManagementApi.md#delete_leech) | **DELETE** /api/v1/admin/leeches/{uuid} | Delete a leech by its uuid
[**gen_leech_config**](LeechManagementApi.md#gen_leech_config) | **GET** /api/v1/admin/leeches/{uuid}/cert | Generate a new config for the leech
[**get_all_leeches**](LeechManagementApi.md#get_all_leeches) | **GET** /api/v1/admin/leeches | Retrieve all leeches
[**get_leech**](LeechManagementApi.md#get_leech) | **GET** /api/v1/admin/leeches/{uuid} | Retrieve a leech by its id
[**update_leech**](LeechManagementApi.md#update_leech) | **PUT** /api/v1/admin/leeches/{uuid} | Update a leech by its id


# **create_leech**
> UuidResponse create_leech(create_leech_request)

Create a leech

Create a leech  The `name` parameter must be unique.  `address` must be a valid address including a scheme and port. Currently only https and http are supported as scheme.

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.create_leech_request import CreateLeechRequest
from kraken_sdk.models.uuid_response import UuidResponse
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.LeechManagementApi(api_client)
    create_leech_request = kraken_sdk.CreateLeechRequest() # CreateLeechRequest | 

    try:
        # Create a leech
        api_response = await api_instance.create_leech(create_leech_request)
        print("The response of LeechManagementApi->create_leech:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling LeechManagementApi->create_leech: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **create_leech_request** | [**CreateLeechRequest**](CreateLeechRequest.md)|  | 

### Return type

[**UuidResponse**](UuidResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Leech got created successfully |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_leech**
> delete_leech(uuid)

Delete a leech by its uuid

Delete a leech by its uuid

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.LeechManagementApi(api_client)
    uuid = 'uuid_example' # str | 

    try:
        # Delete a leech by its uuid
        await api_instance.delete_leech(uuid)
    except Exception as e:
        print("Exception when calling LeechManagementApi->delete_leech: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 

### Return type

void (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Leech got deleted |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **gen_leech_config**
> LeechConfig gen_leech_config(uuid)

Generate a new config for the leech

Generate a new config for the leech

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.leech_config import LeechConfig
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.LeechManagementApi(api_client)
    uuid = 'uuid_example' # str | 

    try:
        # Generate a new config for the leech
        api_response = await api_instance.gen_leech_config(uuid)
        print("The response of LeechManagementApi->gen_leech_config:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling LeechManagementApi->gen_leech_config: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 

### Return type

[**LeechConfig**](LeechConfig.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Newly generated leech cert |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_all_leeches**
> GetAllLeechesResponse get_all_leeches()

Retrieve all leeches

Retrieve all leeches

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.get_all_leeches_response import GetAllLeechesResponse
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.LeechManagementApi(api_client)

    try:
        # Retrieve all leeches
        api_response = await api_instance.get_all_leeches()
        print("The response of LeechManagementApi->get_all_leeches:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling LeechManagementApi->get_all_leeches: %s\n" % e)
```



### Parameters
This endpoint does not need any parameter.

### Return type

[**GetAllLeechesResponse**](GetAllLeechesResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Matched leeches |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_leech**
> SimpleLeech get_leech(uuid)

Retrieve a leech by its id

Retrieve a leech by its id

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.simple_leech import SimpleLeech
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.LeechManagementApi(api_client)
    uuid = 'uuid_example' # str | 

    try:
        # Retrieve a leech by its id
        api_response = await api_instance.get_leech(uuid)
        print("The response of LeechManagementApi->get_leech:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling LeechManagementApi->get_leech: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 

### Return type

[**SimpleLeech**](SimpleLeech.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Matched leeches |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_leech**
> update_leech(uuid, update_leech_request)

Update a leech by its id

Update a leech by its id  All parameter are optional, but at least one of them must be specified.  `address` must be a valid address including a scheme and port. Currently only https and http are supported as scheme.

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.update_leech_request import UpdateLeechRequest
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.LeechManagementApi(api_client)
    uuid = 'uuid_example' # str | 
    update_leech_request = kraken_sdk.UpdateLeechRequest() # UpdateLeechRequest | 

    try:
        # Update a leech by its id
        await api_instance.update_leech(uuid, update_leech_request)
    except Exception as e:
        print("Exception when calling LeechManagementApi->update_leech: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 
 **update_leech_request** | [**UpdateLeechRequest**](UpdateLeechRequest.md)|  | 

### Return type

void (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Leech got updated |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

