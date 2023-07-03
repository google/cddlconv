export type Command = {
  id: JsUint;
} & CommandData &
  Extensible;
export type CommandData =
  | BrowserCommand
  | BrowsingContextCommand
  | InputCommand
  | NetworkCommand
  | ScriptCommand
  | SessionCommand;
export type EmptyParams = Extensible;
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
export type SessionCommand =
  | Session.End
  | Session.New
  | Session.Status
  | Session.Subscribe
  | Session.Unsubscribe;
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
      socksProxy?: string
      /**
       * Must be between `0` and `255`, inclusive.
       */;
      socksVersion?: number;
    };
  } & Extensible;
}
export namespace Session {
  export type SubscriptionRequest = {
    events: [...string[]];
    contexts?: [...BrowsingContext.BrowsingContext[]];
  };
}
export namespace Session {
  export type Status = {
    method: "session.status";
    params: EmptyParams;
  };
}
export namespace Session {
  export type New = {
    method: "session.new";
    params: Session.NewParameters;
  };
}
export namespace Session {
  export type NewParameters = {
    capabilities: Session.CapabilitiesRequest;
  };
}
export namespace Session {
  export type End = {
    method: "session.end";
    params: EmptyParams;
  };
}
export namespace Session {
  export type Subscribe = {
    method: "session.subscribe";
    params: Session.SubscriptionRequest;
  };
}
export namespace Session {
  export type Unsubscribe = {
    method: "session.unsubscribe";
    params: Session.SubscriptionRequest;
  };
}
export type BrowserCommand = Browser.Close;
export namespace Browser {
  export type Close = {
    method: "browser.close";
    params: EmptyParams;
  };
}
export type BrowsingContextCommand =
  | BrowsingContext.CaptureScreenshot
  | BrowsingContext.Close
  | BrowsingContext.Create
  | BrowsingContext.GetTree
  | BrowsingContext.HandleUserPrompt
  | BrowsingContext.Navigate
  | BrowsingContext.Print
  | BrowsingContext.Reload;
export namespace BrowsingContext {
  export type BrowsingContext = string;
}
export namespace BrowsingContext {
  export type Navigation = string;
}
export namespace BrowsingContext {
  export const enum ReadinessState {
    None = "none",
    Interactive = "interactive",
    Complete = "complete",
  }
}
export namespace BrowsingContext {
  export type CaptureScreenshot = {
    method: "browsingContext.captureScreenshot";
    params: BrowsingContext.CaptureScreenshotParameters;
  };
}
export namespace BrowsingContext {
  export type CaptureScreenshotParameters = {
    context: BrowsingContext.BrowsingContext;
  };
}
export namespace BrowsingContext {
  export type Close = {
    method: "browsingContext.close";
    params: BrowsingContext.CloseParameters;
  };
}
export namespace BrowsingContext {
  export type CloseParameters = {
    context: BrowsingContext.BrowsingContext;
  };
}
export namespace BrowsingContext {
  export type Create = {
    method: "browsingContext.create";
    params: BrowsingContext.CreateParameters;
  };
}
export namespace BrowsingContext {
  export const enum CreateType {
    Tab = "tab",
    Window = "window",
  }
}
export namespace BrowsingContext {
  export type CreateParameters = {
    type: BrowsingContext.CreateType;
    referenceContext?: BrowsingContext.BrowsingContext;
  };
}
export namespace BrowsingContext {
  export type GetTree = {
    method: "browsingContext.getTree";
    params: BrowsingContext.GetTreeParameters;
  };
}
export namespace BrowsingContext {
  export type GetTreeParameters = {
    maxDepth?: JsUint;
    root?: BrowsingContext.BrowsingContext;
  };
}
export namespace BrowsingContext {
  export type HandleUserPrompt = {
    method: "browsingContext.handleUserPrompt";
    params: BrowsingContext.HandleUserPromptParameters;
  };
}
export namespace BrowsingContext {
  export type HandleUserPromptParameters = {
    context: BrowsingContext.BrowsingContext;
    accept?: boolean;
    userText?: string;
  };
}
export namespace BrowsingContext {
  export type Navigate = {
    method: "browsingContext.navigate";
    params: BrowsingContext.NavigateParameters;
  };
}
export namespace BrowsingContext {
  export type NavigateParameters = {
    context: BrowsingContext.BrowsingContext;
    url: string;
    wait?: BrowsingContext.ReadinessState;
  };
}
export namespace BrowsingContext {
  export type Print = {
    method: "browsingContext.print";
    params: BrowsingContext.PrintParameters;
  };
}
export namespace BrowsingContext {
  export type PrintParameters = {
    context: BrowsingContext.BrowsingContext
    /**
     * @defaultValue `false`
     */;
    background?: boolean;
    margin?: BrowsingContext.PrintMarginParameters
    /**
     * @defaultValue `"portrait"`
     */;
    orientation?: "portrait" | "landscape";
    page?: BrowsingContext.PrintPageParameters;
    pageRanges?: [...(JsUint | string)[]]
    /**
     * Must be between `0.1` and `2`, inclusive.
     *
     * @defaultValue `1`
     */;
    scale?: number
    /**
     * @defaultValue `true`
     */;
    shrinkToFit?: boolean;
  };
}
export namespace BrowsingContext {
  export type PrintMarginParameters = {
    /**
     * Must be greater than or equal to `0`.
     *
     * @defaultValue `1`
     */
    bottom?: number
    /**
     * Must be greater than or equal to `0`.
     *
     * @defaultValue `1`
     */;
    left?: number
    /**
     * Must be greater than or equal to `0`.
     *
     * @defaultValue `1`
     */;
    right?: number
    /**
     * Must be greater than or equal to `0`.
     *
     * @defaultValue `1`
     */;
    top?: number;
  };
}
export namespace BrowsingContext {
  export type PrintPageParameters = {
    /**
     * Must be greater than or equal to `0`.
     *
     * @defaultValue `27.94`
     */
    height?: number
    /**
     * Must be greater than or equal to `0`.
     *
     * @defaultValue `21.59`
     */;
    width?: number;
  };
}
export namespace BrowsingContext {
  export type Reload = {
    method: "browsingContext.reload";
    params: BrowsingContext.ReloadParameters;
  };
}
export namespace BrowsingContext {
  export type ReloadParameters = {
    context: BrowsingContext.BrowsingContext;
    ignoreCache?: boolean;
    wait?: BrowsingContext.ReadinessState;
  };
}
export namespace BrowsingContext {
  export type SetViewport = {
    method: "browsingContext.setViewport";
    params: BrowsingContext.SetViewportParameters;
  };
}
export namespace BrowsingContext {
  export type SetViewportParameters = {
    context: BrowsingContext.BrowsingContext;
    viewport: BrowsingContext.Viewport | null;
  };
}
export namespace BrowsingContext {
  export type Viewport = {
    width: JsUint;
    height: JsUint;
  };
}
export type NetworkCommand = {};
export namespace Network {
  export type Request = string;
}
export type ScriptCommand =
  | Script.AddPreloadScriptCommand
  | Script.CallFunction
  | Script.Disown
  | Script.Evaluate
  | Script.GetRealms
  | Script.RemovePreloadScriptCommand;
export namespace Script {
  export type Channel = string;
}
export namespace Script {
  export type ChannelValue = {
    type: "channel";
    value: Script.ChannelProperties;
  };
}
export namespace Script {
  export type ChannelProperties = {
    channel: Script.Channel;
    serializationOptions?: Script.SerializationOptions;
    ownership?: Script.ResultOwnership;
  };
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
  export type RemoteReference =
    | Script.SharedReference
    | Script.RemoteObjectReference;
}
export namespace Script {
  export type SharedReference = {
    sharedId: Script.SharedId;
    handle?: Script.Handle;
  } & Extensible;
}
export namespace Script {
  export type RemoteObjectReference = {
    handle: Script.Handle;
    sharedId?: Script.SharedId;
  } & Extensible;
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
    handle?: Script.Handle;
    internalId?: Script.InternalId;
  };
}
export namespace Script {
  export const enum ResultOwnership {
    Root = "root",
    None = "none",
  }
}
export namespace Script {
  export type SerializationOptions = {
    /**
     * @defaultValue `0`
     */
    maxDomDepth?: JsUint | null
    /**
     * @defaultValue `null`
     */;
    maxObjectDepth?: JsUint | null
    /**
     * @defaultValue `"none"`
     */;
    includeShadowTree?: "none" | "open" | "all";
  };
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
  export type RealmTarget = {
    realm: Script.Realm;
  };
}
export namespace Script {
  export type ContextTarget = {
    context: BrowsingContext.BrowsingContext;
    sandbox?: string;
  };
}
export namespace Script {
  export type Target = Script.RealmTarget | Script.ContextTarget;
}
export namespace Script {
  export type AddPreloadScriptCommand = {
    method: "script.addPreloadScript";
    params: Script.AddPreloadScriptParameters;
  };
}
export namespace Script {
  export type AddPreloadScriptParameters = {
    functionDeclaration: string;
    arguments?: [...Script.ChannelValue[]];
    sandbox?: string;
  };
}
export namespace Script {
  export type Disown = {
    method: "script.disown";
    params: Script.DisownParameters;
  };
}
export namespace Script {
  export type DisownParameters = {
    handles: [Script.Handle];
    target: Script.Target;
  };
}
export namespace Script {
  export type CallFunction = {
    method: "script.callFunction";
    params: Script.CallFunctionParameters;
  };
}
export namespace Script {
  export type CallFunctionParameters = {
    functionDeclaration: string;
    awaitPromise: boolean;
    target: Script.Target;
    arguments?: [...Script.ArgumentValue[]];
    resultOwnership?: Script.ResultOwnership;
    serializationOptions?: Script.SerializationOptions;
    this?: Script.ArgumentValue;
  };
}
export namespace Script {
  export type ArgumentValue =
    | Script.RemoteReference
    | Script.LocalValue
    | Script.ChannelValue;
}
export namespace Script {
  export type Evaluate = {
    method: "script.evaluate";
    params: Script.EvaluateParameters;
  };
}
export namespace Script {
  export type EvaluateParameters = {
    expression: string;
    target: Script.Target;
    awaitPromise: boolean;
    resultOwnership?: Script.ResultOwnership;
    serializationOptions?: Script.SerializationOptions;
  };
}
export namespace Script {
  export type GetRealms = {
    method: "script.getRealms";
    params: Script.GetRealmsParameters;
  };
}
export namespace Script {
  export type GetRealmsParameters = {
    context?: BrowsingContext.BrowsingContext;
    type?: Script.RealmType;
  };
}
export namespace Script {
  export type RemovePreloadScriptCommand = {
    method: "script.removePreloadScript";
    params: Script.RemovePreloadScriptParameters;
  };
}
export namespace Script {
  export type RemovePreloadScriptParameters = {
    script: Script.PreloadScript;
  };
}
export type InputCommand = Input.PerformActions | Input.ReleaseActions;
export namespace Input {
  export type ElementOrigin = {
    type: "element";
    element: Script.SharedReference;
  };
}
export namespace Input {
  export type PerformActions = {
    method: "input.performActions";
    params: Input.PerformActionsParameters;
  };
}
export namespace Input {
  export type PerformActionsParameters = {
    context: BrowsingContext.BrowsingContext;
    actions: [...Input.SourceActions[]];
  };
}
export namespace Input {
  export type SourceActions =
    | Input.NoneSourceActions
    | Input.KeySourceActions
    | Input.PointerSourceActions
    | Input.WheelSourceActions;
}
export namespace Input {
  export type NoneSourceActions = {
    type: "none";
    id: string;
    actions: [...Input.NoneSourceAction[]];
  };
}
export namespace Input {
  export type NoneSourceAction = Input.PauseAction;
}
export namespace Input {
  export type KeySourceActions = {
    type: "key";
    id: string;
    actions: [...Input.KeySourceAction[]];
  };
}
export namespace Input {
  export type KeySourceAction =
    | Input.PauseAction
    | Input.KeyDownAction
    | Input.KeyUpAction;
}
export namespace Input {
  export type PointerSourceActions = {
    type: "pointer";
    id: string;
    parameters?: Input.PointerParameters;
    actions: [...Input.PointerSourceAction[]];
  };
}
export namespace Input {
  export const enum PointerType {
    Mouse = "mouse",
    Pen = "pen",
    Touch = "touch",
  }
}
export namespace Input {
  export type PointerParameters = {
    /**
     * @defaultValue `"mouse"`
     */
    pointerType?: Input.PointerType;
  };
}
export namespace Input {
  export type PointerSourceAction =
    | Input.PauseAction
    | Input.PointerDownAction
    | Input.PointerUpAction
    | Input.PointerMoveAction;
}
export namespace Input {
  export type WheelSourceActions = {
    type: "wheel";
    id: string;
    actions: [...Input.WheelSourceAction[]];
  };
}
export namespace Input {
  export type WheelSourceAction = Input.PauseAction | Input.WheelScrollAction;
}
export namespace Input {
  export type PauseAction = {
    type: "pause";
    duration?: JsUint;
  };
}
export namespace Input {
  export type KeyDownAction = {
    type: "keyDown";
    value: string;
  };
}
export namespace Input {
  export type KeyUpAction = {
    type: "keyUp";
    value: string;
  };
}
export namespace Input {
  export type PointerUpAction = {
    type: "pointerUp";
    button: JsUint;
  } & Input.PointerCommonProperties;
}
export namespace Input {
  export type PointerDownAction = {
    type: "pointerDown";
    button: JsUint;
  } & Input.PointerCommonProperties;
}
export namespace Input {
  export type PointerMoveAction = {
    type: "pointerMove";
    x: JsInt;
    y: JsInt;
    duration?: JsUint;
    origin?: Input.Origin;
  } & Input.PointerCommonProperties;
}
export namespace Input {
  export type WheelScrollAction = {
    type: "scroll";
    x: JsInt;
    y: JsInt;
    deltaX: JsInt;
    deltaY: JsInt;
    duration?: JsUint
    /**
     * @defaultValue `"viewport"`
     */;
    origin?: Input.Origin;
  };
}
export namespace Input {
  export type PointerCommonProperties = {
    /**
     * @defaultValue `1`
     */
    width?: JsUint
    /**
     * @defaultValue `1`
     */;
    height?: JsUint
    /**
     * @defaultValue `0`
     */;
    pressure?: number
    /**
     * @defaultValue `0`
     */;
    tangentialPressure?: number
    /**
     * Must be between `0` and `359`, inclusive.
     *
     * @defaultValue `0`
     */;
    twist?: number;
  } & (Input.TiltProperties | Input.AngleProperties);
}
export namespace Input {
  export type AngleProperties = {
    /**
     * @defaultValue `0`
     */
    altitudeAngle?: number
    /**
     * @defaultValue `0`
     */;
    azimuthAngle?: number;
  };
}
export namespace Input {
  export type TiltProperties = {
    /**
     * Must be between `-90` and `90`, inclusive.
     *
     * @defaultValue `0`
     */
    tiltX?: number
    /**
     * Must be between `-90` and `90`, inclusive.
     *
     * @defaultValue `0`
     */;
    tiltY?: number;
  };
}
export namespace Input {
  export type Origin = "viewport" | "pointer" | Input.ElementOrigin;
}
export namespace Input {
  export type ReleaseActions = {
    method: "input.releaseActions";
    params: Input.ReleaseActionsParameters;
  };
}
export namespace Input {
  export type ReleaseActionsParameters = {
    context: BrowsingContext.BrowsingContext;
  };
}
