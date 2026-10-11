export {
  CapabilityConfigurationError,
  CapabilityInvocationError,
  CapabilityProtocolError,
  CapabilityTransportUnavailableError,
} from "./errors.js";
export {
  KEY_VALUE_INTERFACE,
  deleteKey as deleteKvKey,
  get as getKv,
  kv,
  set as setKv,
} from "./kv.js";
export {
  OBJECT_STORE_INTERFACE,
  deleteObject,
  get as getObject,
  head as headObject,
  list as listObjects,
  objects,
  put as putObject,
} from "./objects.js";
export type { ObjectInfo, StoredObject } from "./objects.js";
export type { CapabilityCall, CapabilityTransport } from "./transport.js";
