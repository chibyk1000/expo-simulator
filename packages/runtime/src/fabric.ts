export interface FabricNode {
  tag: number;
  viewName: string;
  rootTag: number;
  props: Record<string, any>;
  children: FabricNode[];
  instanceHandle?: any;
}

export function setupFabricUIManager(
  sendRequest: (method: string, params: any) => void
) {
  const g = (typeof globalThis !== 'undefined' ? globalThis : typeof window !== 'undefined' ? window : {}) as any;

  const nodeMap = new Map<number, FabricNode>();
  let nextCallbackId = 1;
  const measureCallbacks = new Map<number, (left: number, top: number, width: number, height: number, pageX: number, pageY: number) => void>();

  const fabricUIManager = {
    createNode(
      tag: number,
      viewName: string,
      rootTag: number,
      props: Record<string, any>,
      instanceHandle: any
    ): FabricNode {
      const node: FabricNode = {
        tag,
        viewName,
        rootTag,
        props: props || {},
        children: [],
        instanceHandle,
      };
      nodeMap.set(tag, node);
      sendRequest('ui.createNode', {
        id: tag,
        viewName,
        rootTag,
        props: props || {},
      });
      return node;
    },

    cloneNodeWithNewProps(node: FabricNode, newProps: Record<string, any>): FabricNode {
      const cloned: FabricNode = {
        ...node,
        props: { ...node.props, ...newProps },
      };
      nodeMap.set(cloned.tag, cloned);
      sendRequest('ui.updateProps', {
        id: cloned.tag,
        props: cloned.props,
      });
      return cloned;
    },

    cloneNodeWithNewChildren(node: FabricNode, newChildren: FabricNode[]): FabricNode {
      const cloned: FabricNode = {
        ...node,
        children: newChildren || [],
      };
      nodeMap.set(cloned.tag, cloned);
      sendRequest('ui.setChildren', {
        id: cloned.tag,
        children: cloned.children.map((c) => c.tag),
      });
      return cloned;
    },

    cloneNodeWithNewChildrenAndProps(
      node: FabricNode,
      newProps: Record<string, any>,
      newChildren: FabricNode[]
    ): FabricNode {
      const cloned: FabricNode = {
        ...node,
        props: { ...node.props, ...newProps },
        children: newChildren || [],
      };
      nodeMap.set(cloned.tag, cloned);
      sendRequest('ui.updateProps', {
        id: cloned.tag,
        props: cloned.props,
      });
      sendRequest('ui.setChildren', {
        id: cloned.tag,
        children: cloned.children.map((c) => c.tag),
      });
      return cloned;
    },

    appendChild(parentNode: FabricNode, childNode: FabricNode): FabricNode {
      parentNode.children.push(childNode);
      sendRequest('ui.appendChild', {
        parentId: parentNode.tag,
        childId: childNode.tag,
      });
      return parentNode;
    },

    completeRoot(rootTag: number, childNodes: FabricNode[]) {
      sendRequest('ui.completeRoot', {
        rootTag,
        children: (childNodes || []).map((c) => c.tag),
      });
    },

    measure(
      node: FabricNode,
      callback: (left: number, top: number, width: number, height: number, pageX: number, pageY: number) => void
    ) {
      const callbackId = nextCallbackId++;
      measureCallbacks.set(callbackId, callback);
      sendRequest('ui.measure', {
        id: node.tag,
        callbackId,
      });
    },

    findShadowNodeByTag_DEPRECATED(tag: number): FabricNode | undefined {
      return nodeMap.get(tag);
    },

    setNativeProps(node: FabricNode, newProps: Record<string, any>) {
      this.cloneNodeWithNewProps(node, newProps);
    },

    dispatchCommand(node: FabricNode, commandName: string, args: any[]) {
      sendRequest('ui.dispatchCommand', {
        id: node.tag,
        commandName,
        args,
      });
    },
  };

  g.nativeFabricUIManager = fabricUIManager;

  return {
    handleMeasureResult(params: { callbackId: number; x: number; y: number; width: number; height: number; pageX: number; pageY: number }) {
      const cb = measureCallbacks.get(params.callbackId);
      if (cb) {
        measureCallbacks.delete(params.callbackId);
        cb(params.x, params.y, params.width, params.height, params.pageX, params.pageY);
      }
    },
    getNode(tag: number): FabricNode | undefined {
      return nodeMap.get(tag);
    },
  };
}
