# kraken_sdk.AuthenticationApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**finish_auth**](AuthenticationApi.md#finish_auth) | **POST** /api/v1/auth/finishAuth | Finishes the authentication with a security key
[**finish_register**](AuthenticationApi.md#finish_register) | **POST** /api/v1/auth/finishRegister | Finish the registration of a security key
[**login**](AuthenticationApi.md#login) | **POST** /api/v1/auth/login | Login to kraken
[**logout**](AuthenticationApi.md#logout) | **GET** /api/v1/auth/logout | Log out of this session
[**start_auth**](AuthenticationApi.md#start_auth) | **POST** /api/v1/auth/startAuth | Starts the authentication with a security key
[**start_register**](AuthenticationApi.md#start_register) | **POST** /api/v1/auth/startRegister | Start the registration of a security key
[**test**](AuthenticationApi.md#test) | **GET** /api/v1/auth/test | Test the current login state


# **finish_auth**
> finish_auth(body)

Finishes the authentication with a security key

Finishes the authentication with a security key  Use `startAuth` to retrieve the challenge response data.

### Example

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


# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AuthenticationApi(api_client)
    body = None # object | 

    try:
        # Finishes the authentication with a security key
        await api_instance.finish_auth(body)
    except Exception as e:
        print("Exception when calling AuthenticationApi->finish_auth: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **body** | **object**|  | 

### Return type

void (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | 2FA Authentication finished |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **finish_register**
> finish_register(finish_register_request)

Finish the registration of a security key

Finish the registration of a security key  Use `startRegister` to retrieve the challenge response data.

### Example

```python
import time
import os
import kraken_sdk
from kraken_sdk.models.finish_register_request import FinishRegisterRequest
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AuthenticationApi(api_client)
    finish_register_request = kraken_sdk.FinishRegisterRequest() # FinishRegisterRequest | 

    try:
        # Finish the registration of a security key
        await api_instance.finish_register(finish_register_request)
    except Exception as e:
        print("Exception when calling AuthenticationApi->finish_register: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **finish_register_request** | [**FinishRegisterRequest**](FinishRegisterRequest.md)|  | 

### Return type

void (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | 2FA Key registration finished |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **login**
> login(login_request)

Login to kraken

Login to kraken

### Example

```python
import time
import os
import kraken_sdk
from kraken_sdk.models.login_request import LoginRequest
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)


# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AuthenticationApi(api_client)
    login_request = kraken_sdk.LoginRequest() # LoginRequest | 

    try:
        # Login to kraken
        await api_instance.login(login_request)
    except Exception as e:
        print("Exception when calling AuthenticationApi->login: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **login_request** | [**LoginRequest**](LoginRequest.md)|  | 

### Return type

void (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Login successful |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **logout**
> logout()

Log out of this session

Log out of this session  Logs a logged-in user out of his session.

### Example

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


# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AuthenticationApi(api_client)

    try:
        # Log out of this session
        await api_instance.logout()
    except Exception as e:
        print("Exception when calling AuthenticationApi->logout: %s\n" % e)
```



### Parameters
This endpoint does not need any parameter.

### Return type

void (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Logout successful |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **start_auth**
> object start_auth()

Starts the authentication with a security key

Starts the authentication with a security key  Use the `login` endpoint before calling this one.  Proceed with `finishAuth`.

### Example

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


# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AuthenticationApi(api_client)

    try:
        # Starts the authentication with a security key
        api_response = await api_instance.start_auth()
        print("The response of AuthenticationApi->start_auth:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthenticationApi->start_auth: %s\n" % e)
```



### Parameters
This endpoint does not need any parameter.

### Return type

**object**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | 2FA Authentication started |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **start_register**
> object start_register()

Start the registration of a security key

Start the registration of a security key  Proceed to the `finishRegister` endpoint.

### Example

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


# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AuthenticationApi(api_client)

    try:
        # Start the registration of a security key
        api_response = await api_instance.start_register()
        print("The response of AuthenticationApi->start_register:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AuthenticationApi->start_register: %s\n" % e)
```



### Parameters
This endpoint does not need any parameter.

### Return type

**object**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | 2FA Key registration started |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **test**
> test()

Test the current login state

Test the current login state  You can use this endpoint to test the current login state of your client.  If logged in, a 200 without a body is returned.

### Example

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


# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AuthenticationApi(api_client)

    try:
        # Test the current login state
        await api_instance.test()
    except Exception as e:
        print("Exception when calling AuthenticationApi->test: %s\n" % e)
```



### Parameters
This endpoint does not need any parameter.

### Return type

void (empty response body)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Logged in |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

