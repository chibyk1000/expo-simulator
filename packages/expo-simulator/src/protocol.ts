export interface CreateNodeParams {
  id: number;
  viewName: string;
  rootTag: number;
  props: Record<string, any>;
}

export interface CloneNodeParams {
  id: number;
  newId: number;
  props?: Record<string, any>;
  children?: number[];
}

export interface UpdatePropsParams {
  id: number;
  props: Record<string, any>;
}

export interface SetChildrenParams {
  id: number;
  children: number[];
}

export interface CompleteRootParams {
  rootTag: number;
  children: number[];
}

export interface MeasureParams {
  id: number;
  callbackId: number;
}

export interface DispatchEventParams {
  targetId: number;
  eventName: string;
  payload: Record<string, any>;
}

export interface MeasureResultParams {
  callbackId: number;
  x: number;
  y: number;
  width: number;
  height: number;
  pageX: number;
  pageY: number;
}
