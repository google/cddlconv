export type Message = CommandResponse | ErrorResponse | Event;
export type CommandResponse = {
  id: JsUint;
  result: ResultData;
} & Extensible;
export type ErrorResponse = {
  id: JsUint | null;
  error: ErrorCode;
  message: string;
  stacktrace?: string;
} & Extensible;
export type ResultData =
  | BrowsingContextResult
  | EmptyResult
  | NetworkResult
  | ScriptResult
  | SessionResult;
export type EmptyResult = Extensible;
export type Event = EventData & Extensible;
export type EventData =
  | BrowsingContextEvent
  | LogEvent
  | NetworkEvent
  | ScriptEvent;
export type Extensible = {
  [key: string]: any;
};
/**
 * Must be between `-9007199254740991` and `9007199254740991`, inclusive.
 */
export type JsInt = number;
/**
 * Must be between `0` and `9007199254740991`, inclusive.
 */
export type JsUint = number;
export const enum ErrorCode {
  InvalidArgument = "invalid argument",
  InvalidSessionId = "invalid session id",
  NoSuchAlert = "no such alert",
  NoSuchFrame = "no such frame",
  NoSuchHandle = "no such handle",
  NoSuchNode = "no such node",
  NoSuchScript = "no such script",
  SessionNotCreated = "session not created",
  UnableToCaptureScreen = "unable to capture screen",
  UnableToCloseBrowser = "unable to close browser",
  UnknownCommand = "unknown command",
  UnknownError = "unknown error",
  UnsupportedOperation = "unsupported operation",
}
export type SessionResult = Session.NewResult | Session.StatusResult;
export namespace Session {
  export type CapabilitiesRequest = {
    alwaysMatch?: Session.CapabilityRequest;
    firstMatch?: [...Session.CapabilityRequest[]];
  };
}
export namespace Session {
  export type CapabilityRequest = {
    acceptInsecureCerts?: boolean;
    browserName?: string;
    browserVersion?: string;
    platformName?: string;
    proxy?: {
      proxyType?: "pac" | "direct" | "autodetect" | "system" | "manual";
      proxyAutoconfigUrl?: string;
      ftpProxy?: string;
      httpProxy?: string;
      noProxy?: [...string[]];
      sslProxy?: string;
      socksProxy?: string;
      /**
       * Must be between `0` and `255`, inclusive.
       */
      socksVersion?: number;
    };
  } & Extensible;
}
export namespace Session {
  export type StatusResult = {
    ready: boolean;
    message: string;
  };
}
export namespace Session {
  export type NewResult = {
    sessionId: string;
    capabilities: {
      acceptInsecureCerts: boolean;
      browserName: string;
      browserVersion: string;
      platformName: string;
      proxy: {
        proxyType?: "pac" | "direct" | "autodetect" | "system" | "manual";
        proxyAutoconfigUrl?: string;
        ftpProxy?: string;
        httpProxy?: string;
        noProxy?: [...string[]];
        sslProxy?: string;
        socksProxy?: string;
        /**
         * Must be between `0` and `255`, inclusive.
         */
        socksVersion?: number;
      };
      setWindowRect: boolean;
    } & Extensible;
  };
}
export type BrowsingContextResult =
  | BrowsingContext.CaptureScreenshotResult
  | BrowsingContext.CreateResult
  | BrowsingContext.GetTreeResult
  | BrowsingContext.NavigateResult
  | BrowsingContext.PrintResult;
export type BrowsingContextEvent =
  | BrowsingContext.ContextCreated
  | BrowsingContext.ContextDestroyed
  | BrowsingContext.NavigationStarted
  | BrowsingContext.FragmentNavigated
  | BrowsingContext.DomContentLoaded
  | BrowsingContext.Load
  | BrowsingContext.DownloadWillBegin
  | BrowsingContext.NavigationAborted
  | BrowsingContext.NavigationFailed
  | BrowsingContext.UserPromptClosed
  | BrowsingContext.UserPromptOpened;
export namespace BrowsingContext {
  export type BrowsingContext = string;
}
export namespace BrowsingContext {
  export type InfoList = [...BrowsingContext.Info[]];
}
export namespace BrowsingContext {
  export type Info = {
    context: BrowsingContext.BrowsingContext;
    url: string;
    children: BrowsingContext.InfoList | null;
    parent?: BrowsingContext.BrowsingContext | null;
  };
}
export namespace BrowsingContext {
  export type Navigation = string;
}
export namespace BrowsingContext {
  export type NavigationInfo = {
    context: BrowsingContext.BrowsingContext;
    navigation: BrowsingContext.Navigation | null;
    timestamp: JsUint;
    url: string;
  };
}
export namespace BrowsingContext {
  export type CaptureScreenshotResult = {
    data: string;
  };
}
export namespace BrowsingContext {
  export type CreateResult = {
    context: BrowsingContext.BrowsingContext;
  };
}
export namespace BrowsingContext {
  export type GetTreeResult = {
    contexts: BrowsingContext.InfoList;
  };
}
export namespace BrowsingContext {
  export type NavigateResult = {
    navigation: BrowsingContext.Navigation | null;
    url: string;
  };
}
export namespace BrowsingContext {
  export type PrintResult = {
    data: string;
  };
}
export namespace BrowsingContext {
  export type ContextCreated = {
    method: "browsingContext.contextCreated";
    params: BrowsingContext.Info;
  };
}
export namespace BrowsingContext {
  export type ContextDestroyed = {
    method: "browsingContext.contextDestroyed";
    params: BrowsingContext.Info;
  };
}
export namespace BrowsingContext {
  export type NavigationStarted = {
    method: "browsingContext.navigationStarted";
    params: BrowsingContext.NavigationInfo;
  };
}
export namespace BrowsingContext {
  export type FragmentNavigated = {
    method: "browsingContext.fragmentNavigated";
    params: BrowsingContext.NavigationInfo;
  };
}
export namespace BrowsingContext {
  export type DomContentLoaded = {
    method: "browsingContext.domContentLoaded";
    params: BrowsingContext.NavigationInfo;
  };
}
export namespace BrowsingContext {
  export type Load = {
    method: "browsingContext.load";
    params: BrowsingContext.NavigationInfo;
  };
}
export namespace BrowsingContext {
  export type DownloadWillBegin = {
    method: "browsingContext.downloadWillBegin";
    params: BrowsingContext.NavigationInfo;
  };
}
export namespace BrowsingContext {
  export type NavigationAborted = {
    method: "browsingContext.navigationAborted";
    params: BrowsingContext.NavigationInfo;
  };
}
export namespace BrowsingContext {
  export type NavigationFailed = {
    method: "browsingContext.navigationFailed";
    params: BrowsingContext.NavigationInfo;
  };
}
export namespace BrowsingContext {
  export type UserPromptClosed = {
    method: "browsingContext.userPromptClosed";
    params: BrowsingContext.UserPromptClosedParameters;
  };
}
export namespace BrowsingContext {
  export type UserPromptClosedParameters = {
    context: BrowsingContext.BrowsingContext;
    accepted: boolean;
    userText?: string;
  };
}
export namespace BrowsingContext {
  export type UserPromptOpened = {
    method: "browsingContext.userPromptOpened";
    params: BrowsingContext.UserPromptOpenedParameters;
  };
}
export namespace BrowsingContext {
  export type UserPromptOpenedParameters = {
    context: BrowsingContext.BrowsingContext;
    type: "alert" | "confirm" | "prompt" | "beforeunload";
    message: string;
  };
}
export type NetworkResult = {};
export type NetworkEvent =
  | Network.BeforeRequestSent
  | Network.FetchError
  | Network.ResponseStarted
  | Network.ResponseCompleted;
export namespace Network {
  export type BaseParameters = {
    context: BrowsingContext.BrowsingContext | null;
    navigation: BrowsingContext.Navigation | null;
    redirectCount: JsUint;
    request: Network.RequestData;
    timestamp: JsUint;
  };
}
export namespace Network {
  export type Cookie = {
    name: string;
    value?: string;
    binaryValue?: [number];
    domain: string;
    path: string;
    expires?: JsUint;
    size: JsUint;
    httpOnly: boolean;
    secure: boolean;
    sameSite: "strict" | "lax" | "none";
  };
}
export namespace Network {
  export type FetchTimingInfo = {
    timeOrigin: number;
    requestTime: number;
    redirectStart: number;
    redirectEnd: number;
    fetchStart: number;
    dnsStart: number;
    dnsEnd: number;
    connectStart: number;
    connectEnd: number;
    tlsStart: number;
    requestStart: number;
    responseStart: number;
    responseEnd: number;
  };
}
export namespace Network {
  export type Header = {
    name: string;
    value?: string;
    binaryValue?: [number];
  };
}
export namespace Network {
  export type Initiator = {
    type: "parser" | "script" | "preflight" | "other";
    columnNumber?: JsUint;
    lineNumber?: JsUint;
    stackTrace?: Script.StackTrace;
    request?: Network.Request;
  };
}
export namespace Network {
  export type Request = string;
}
export namespace Network {
  export type RequestData = {
    request: Network.Request;
    url: string;
    method: string;
    headers: [...Network.Header[]];
    cookies: [...Network.Cookie[]];
    headersSize: JsUint;
    bodySize: JsUint | null;
    timings: Network.FetchTimingInfo;
  };
}
export namespace Network {
  export type ResponseContent = {
    size: JsUint;
  };
}
export namespace Network {
  export type ResponseData = {
    url: string;
    protocol: string;
    status: JsUint;
    statusText: string;
    fromCache: boolean;
    headers: [...Network.Header[]];
    mimeType: string;
    bytesReceived: JsUint;
    headersSize: JsUint | null;
    bodySize: JsUint | null;
    content: Network.ResponseContent;
  };
}
export namespace Network {
  export type BeforeRequestSent = {
    method: "network.beforeRequestSent";
    params: Network.BeforeRequestSentParameters;
  };
}
export namespace Network {
  export type BeforeRequestSentParameters = Network.BaseParameters & {
    initiator: Network.Initiator;
  };
}
export namespace Network {
  export type FetchError = {
    method: "network.fetchError";
    params: Network.FetchErrorParameters;
  };
}
export namespace Network {
  export type FetchErrorParameters = Network.BaseParameters & {
    errorText: string;
  };
}
export namespace Network {
  export type ResponseCompleted = {
    method: "network.responseCompleted";
    params: Network.ResponseCompletedParameters;
  };
}
export namespace Network {
  export type ResponseCompletedParameters = Network.BaseParameters & {
    response: Network.ResponseData;
  };
}
export namespace Network {
  export type ResponseStarted = {
    method: "network.responseStarted";
    params: Network.ResponseStartedParameters;
  };
}
export namespace Network {
  export type ResponseStartedParameters = Network.BaseParameters & {
    response: Network.ResponseData;
  };
}
export type ScriptResult =
  | Script.AddPreloadScriptResult
  | Script.EvaluateResult
  | Script.GetRealmsResult;
export type ScriptEvent = Script.RealmCreated | Script.RealmDestroyed;
export namespace Script {
  export type Channel = string;
}
export namespace Script {
  export type EvaluateResult =
    | Script.EvaluateResultSuccess
    | Script.EvaluateResultException;
}
export namespace Script {
  export type EvaluateResultSuccess = {
    type: "success";
    result: Script.RemoteValue;
    realm: Script.Realm;
  };
}
export namespace Script {
  export type EvaluateResultException = {
    type: "exception";
    exceptionDetails: Script.ExceptionDetails;
    realm: Script.Realm;
  };
}
export namespace Script {
  export type ExceptionDetails = {
    columnNumber: JsUint;
    exception: Script.RemoteValue;
    lineNumber: JsUint;
    stackTrace: Script.StackTrace;
    text: string;
  };
}
export namespace Script {
  export type Handle = string;
}
export namespace Script {
  export type LocalValue =
    | Script.PrimitiveProtocolValue
    | Script.ArrayLocalValue
    | Script.DateLocalValue
    | Script.MapLocalValue
    | Script.ObjectLocalValue
    | Script.RegExpLocalValue
    | Script.SetLocalValue;
}
export namespace Script {
  export type ListLocalValue = [...Script.LocalValue[]];
}
export namespace Script {
  export type ArrayLocalValue = {
    type: "array";
    value: Script.ListLocalValue;
  };
}
export namespace Script {
  export type DateLocalValue = {
    type: "date";
    value: string;
  };
}
export namespace Script {
  export type MappingLocalValue = [
    ...[Script.LocalValue | string, Script.LocalValue][]
  ];
}
export namespace Script {
  export type MapLocalValue = {
    type: "map";
    value: Script.MappingLocalValue;
  };
}
export namespace Script {
  export type ObjectLocalValue = {
    type: "object";
    value: Script.MappingLocalValue;
  };
}
export namespace Script {
  export type RegExpValue = {
    pattern: string;
    flags?: string;
  };
}
export namespace Script {
  export type RegExpLocalValue = {
    type: "regexp";
    value: Script.RegExpValue;
  };
}
export namespace Script {
  export type SetLocalValue = {
    type: "set";
    value: Script.ListLocalValue;
  };
}
export namespace Script {
  export type PreloadScript = string;
}
export namespace Script {
  export type Realm = string;
}
export namespace Script {
  export type PrimitiveProtocolValue =
    | Script.UndefinedValue
    | Script.NullValue
    | Script.StringValue
    | Script.NumberValue
    | Script.BooleanValue
    | Script.BigIntValue;
}
export namespace Script {
  export type UndefinedValue = {
    type: "undefined";
  };
}
export namespace Script {
  export type NullValue = {
    type: "null";
  };
}
export namespace Script {
  export type StringValue = {
    type: "string";
    value: string;
  };
}
export namespace Script {
  export type SpecialNumber = "NaN" | "-0" | "Infinity" | "-Infinity";
}
export namespace Script {
  export type NumberValue = {
    type: "number";
    value: number | Script.SpecialNumber;
  };
}
export namespace Script {
  export type BooleanValue = {
    type: "boolean";
    value: boolean;
  };
}
export namespace Script {
  export type BigIntValue = {
    type: "bigint";
    value: string;
  };
}
export namespace Script {
  export type RealmInfo =
    | Script.WindowRealmInfo
    | Script.DedicatedWorkerRealmInfo
    | Script.SharedWorkerRealmInfo
    | Script.ServiceWorkerRealmInfo
    | Script.WorkerRealmInfo
    | Script.PaintWorkletRealmInfo
    | Script.AudioWorkletRealmInfo
    | Script.WorkletRealmInfo;
}
export namespace Script {
  export type BaseRealmInfo = {
    realm: Script.Realm;
    origin: string;
  };
}
export namespace Script {
  export type WindowRealmInfo = Script.BaseRealmInfo & {
    type: "window";
    context: BrowsingContext.BrowsingContext;
    sandbox?: string;
  };
}
export namespace Script {
  export type DedicatedWorkerRealmInfo = Script.BaseRealmInfo & {
    type: "dedicated-worker";
  };
}
export namespace Script {
  export type SharedWorkerRealmInfo = Script.BaseRealmInfo & {
    type: "shared-worker";
  };
}
export namespace Script {
  export type ServiceWorkerRealmInfo = Script.BaseRealmInfo & {
    type: "service-worker";
  };
}
export namespace Script {
  export type WorkerRealmInfo = Script.BaseRealmInfo & {
    type: "worker";
  };
}
export namespace Script {
  export type PaintWorkletRealmInfo = Script.BaseRealmInfo & {
    type: "paint-worklet";
  };
}
export namespace Script {
  export type AudioWorkletRealmInfo = Script.BaseRealmInfo & {
    type: "audio-worklet";
  };
}
export namespace Script {
  export type WorkletRealmInfo = Script.BaseRealmInfo & {
    type: "worklet";
  };
}
export namespace Script {
  export type RealmType =
    | "window"
    | "dedicated-worker"
    | "shared-worker"
    | "service-worker"
    | "worker"
    | "paint-worklet"
    | "audio-worklet"
    | "worklet";
}
export namespace Script {
  export type RemoteValue =
    | Script.PrimitiveProtocolValue
    | Script.SymbolRemoteValue
    | Script.ArrayRemoteValue
    | Script.ObjectRemoteValue
    | Script.FunctionRemoteValue
    | Script.RegExpRemoteValue
    | Script.DateRemoteValue
    | Script.MapRemoteValue
    | Script.SetRemoteValue
    | Script.WeakMapRemoteValue
    | Script.WeakSetRemoteValue
    | Script.IteratorRemoteValue
    | Script.GeneratorRemoteValue
    | Script.ErrorRemoteValue
    | Script.ProxyRemoteValue
    | Script.PromiseRemoteValue
    | Script.TypedArrayRemoteValue
    | Script.ArrayBufferRemoteValue
    | Script.NodeListRemoteValue
    | Script.HtmlCollectionRemoteValue
    | Script.NodeRemoteValue
    | Script.WindowProxyRemoteValue;
}
export namespace Script {
  export type InternalId = JsUint;
}
export namespace Script {
  export type ListRemoteValue = [...Script.RemoteValue[]];
}
export namespace Script {
  export type MappingRemoteValue = [
    ...[Script.RemoteValue | string, Script.RemoteValue][]
  ];
}
export namespace Script {
  export type SymbolRemoteValue = {
    type: "symbol";
    handle?: Script.Handle;
    internalId?: Script.InternalId;
  };
}
export namespace Script {
  export type ArrayRemoteValue = {
    type: "array";
    handle?: Script.Handle;
    internalId?: Script.InternalId;
    value?: Script.ListRemoteValue;
  };
}
export namespace Script {
  export type ObjectRemoteValue = {
    type: "object";
    handle?: Script.Handle;
    internalId?: Script.InternalId;
    value?: Script.MappingRemoteValue;
  };
}
export namespace Script {
  export type FunctionRemoteValue = {
    type: "function";
    handle?: Script.Handle;
    internalId?: Script.InternalId;
  };
}
export namespace Script {
  export type RegExpRemoteValue = {
    handle?: Script.Handle;
    internalId?: Script.InternalId;
  } & Script.RegExpLocalValue;
}
export namespace Script {
  export type DateRemoteValue = {
    handle?: Script.Handle;
    internalId?: Script.InternalId;
  } & Script.DateLocalValue;
}
export namespace Script {
  export type MapRemoteValue = {
    type: "map";
    handle?: Script.Handle;
    internalId?: Script.InternalId;
    value?: Script.MappingRemoteValue;
  };
}
export namespace Script {
  export type SetRemoteValue = {
    type: "set";
    handle?: Script.Handle;
    internalId?: Script.InternalId;
    value?: Script.ListRemoteValue;
  };
}
export namespace Script {
  export type WeakMapRemoteValue = {
    type: "weakmap";
    handle?: Script.Handle;
    internalId?: Script.InternalId;
  };
}
export namespace Script {
  export type WeakSetRemoteValue = {
    type: "weakset";
    handle?: Script.Handle;
    internalId?: Script.InternalId;
  };
}
export namespace Script {
  export type IteratorRemoteValue = {
    type: "iterator";
    handle?: Script.Handle;
    internalId?: Script.InternalId;
  };
}
export namespace Script {
  export type GeneratorRemoteValue = {
    type: "generator";
    handle?: Script.Handle;
    internalId?: Script.InternalId;
  };
}
export namespace Script {
  export type ErrorRemoteValue = {
    type: "error";
    handle?: Script.Handle;
    internalId?: Script.InternalId;
  };
}
export namespace Script {
  export type ProxyRemoteValue = {
    type: "proxy";
    handle?: Script.Handle;
    internalId?: Script.InternalId;
  };
}
export namespace Script {
  export type PromiseRemoteValue = {
    type: "promise";
    handle?: Script.Handle;
    internalId?: Script.InternalId;
  };
}
export namespace Script {
  export type TypedArrayRemoteValue = {
    type: "typedarray";
    handle?: Script.Handle;
    internalId?: Script.InternalId;
  };
}
export namespace Script {
  export type ArrayBufferRemoteValue = {
    type: "arraybuffer";
    handle?: Script.Handle;
    internalId?: Script.InternalId;
  };
}
export namespace Script {
  export type NodeListRemoteValue = {
    type: "nodelist";
    handle?: Script.Handle;
    internalId?: Script.InternalId;
    value?: Script.ListRemoteValue;
  };
}
export namespace Script {
  export type HtmlCollectionRemoteValue = {
    type: "htmlcollection";
    handle?: Script.Handle;
    internalId?: Script.InternalId;
    value?: Script.ListRemoteValue;
  };
}
export namespace Script {
  export type NodeRemoteValue = {
    type: "node";
    sharedId?: Script.SharedId;
    handle?: Script.Handle;
    internalId?: Script.InternalId;
    value?: Script.NodeProperties;
  };
}
export namespace Script {
  export type NodeProperties = {
    nodeType: JsUint;
    childNodeCount: JsUint;
    attributes?: {
      [key: string]: string;
    };
    children?: [...Script.NodeRemoteValue[]];
    localName?: string;
    mode?: "open" | "closed";
    namespaceURI?: string;
    nodeValue?: string;
    shadowRoot?: Script.NodeRemoteValue | null;
  };
}
export namespace Script {
  export type WindowProxyRemoteValue = {
    type: "window";
    value: Script.WindowProxyProperties;
    handle?: Script.Handle;
    internalId?: Script.InternalId;
  };
}
export namespace Script {
  export type WindowProxyProperties = {
    context: BrowsingContext.BrowsingContext;
  };
}
export namespace Script {
  export const enum ResultOwnership {
    Root = "root",
    None = "none",
  }
}
export namespace Script {
  export type SharedId = string;
}
export namespace Script {
  export type StackFrame = {
    columnNumber: JsUint;
    functionName: string;
    lineNumber: JsUint;
    url: string;
  };
}
export namespace Script {
  export type StackTrace = {
    callFrames: [...Script.StackFrame[]];
  };
}
export namespace Script {
  export type Source = {
    realm: Script.Realm;
    context?: BrowsingContext.BrowsingContext;
  };
}
export namespace Script {
  export type AddPreloadScriptResult = {
    script: Script.PreloadScript;
  };
}
export namespace Script {
  export type GetRealmsResult = {
    realms: [...Script.RealmInfo[]];
  };
}
export namespace Script {
  export type Message = {
    method: "script.message";
    params: Script.MessageParameters;
  };
}
export namespace Script {
  export type MessageParameters = {
    channel: Script.Channel;
    data: Script.RemoteValue;
    source: Script.Source;
  };
}
export namespace Script {
  export type RealmCreated = {
    method: "script.realmCreated";
    params: Script.RealmInfo;
  };
}
export namespace Script {
  export type RealmDestroyed = {
    method: "script.realmDestoyed";
    params: Script.RealmDestroyedParameters;
  };
}
export namespace Script {
  export type RealmDestroyedParameters = {
    realm: Script.Realm;
  };
}
export type LogEvent = Log.EntryAdded;
export namespace Log {
  export const enum Level {
    Debug = "debug",
    Info = "info",
    Warn = "warn",
    Error = "error",
  }
}
export namespace Log {
  export type Entry =
    | Log.GenericLogEntry
    | Log.ConsoleLogEntry
    | Log.JavascriptLogEntry;
}
export namespace Log {
  export type BaseLogEntry = {
    level: Log.Level;
    source: Script.Source;
    text: string | null;
    timestamp: JsUint;
    stackTrace?: Script.StackTrace;
  };
}
export namespace Log {
  export type GenericLogEntry = Log.BaseLogEntry & {
    type: string;
  };
}
export namespace Log {
  export type ConsoleLogEntry = Log.BaseLogEntry & {
    type: "console";
    method: string;
    args: [...Script.RemoteValue[]];
  };
}
export namespace Log {
  export type JavascriptLogEntry = Log.BaseLogEntry & {
    type: "javascript";
  };
}
export namespace Log {
  export type EntryAdded = {
    method: "log.entryAdded";
    params: Log.Entry;
  };
}
