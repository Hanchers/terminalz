// ---- Shared types used across Vue components ----

export interface Connection {
  id: number
  name: string
  host: string
  port: number
  username: string
  password: string
  group_id: number
  remark: string
  auto_snippet_id: number
}

export interface Group {
  id: number
  parent_id: number
  name: string
  remark: string
}

export interface Tag {
  id: number
  name: string
  color: string
}

export interface SshKey {
  id: number
  name: string
  key_type: string
  username: string
  password: string
  private_key: string
  host: string
  remark: string
}

export interface PortForward {
  id: number
  name: string
  connection_id: number
  local_port: number
  remote_host: string
  remote_port: number
  direction: string
  enabled: boolean
  remark: string
}

export interface Snippet {
  id: number
  name: string
  content: string
  language: string
  is_favorite: boolean
  remark: string
}

export interface FlatOption {
  id: number
  label: string
  disabled?: boolean
}

export interface HostDialogState {
  visible: boolean
  editingId: number
  name: string
  host: string
  port: number
  username: string
  password: string
  groupId: number
  tagIds: number[]
  remark: string
  autoSnippetId: number
}
