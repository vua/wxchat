import {defineStore} from 'pinia'
import {ref} from "vue"
import {TableInstance} from "element-plus";

export interface Member {
    NickName: string
    RemarkName: string
    UserName: string
    PYQuanPin: string
    VerifyFlag: number
}

export interface User {
    NickName: string
    UserName: string
    PYQuanPin: string
}

export interface Group {
    Id: string
    Name: string
    Members: Member[]
    Operator: string
}

export interface AutoReplyRule {
    Id: string
    Name: string
    Group: Group
    Reply: Reply[]
    Status: boolean
}


export interface Reply {
    ReplyType: string
    Template: Template
    OpenAi: OpenAi
}

export interface Template {
    Keywords: string[]
    Content: string
    Keyword: string
    InputVisible: boolean
}

export interface OpenAi {
    Id: string
    Name: string
    Source: string
    Token: string
    Model: string
    Prompt: string
}

export interface OpenAiConfig {
    Source: string,
    Url: string,
    Model: string[]
}

export interface ScheduledRule {
    Id: string
    Name: string
    Cron: string
    Group: Group
    Reply: Reply[]
    Status: boolean
}

export const globalStore = defineStore('global', () => {
    const loading_completed = ref<boolean>(false)
    const members = ref<Member[]>([])
    const select_members = ref<Member[]>([])
    const user = ref<User>()
    const groups = ref<Group[]>([])
    const auto_reply_rules = ref<AutoReplyRule[]>([])
    const scheduled_rules = ref<ScheduledRule[]>([])
    const selectable = ref<boolean>(true)
    const member_table = ref<TableInstance>()
    const openai_list = ref<OpenAi[]>([])
    const openai_config_list = ref<OpenAiConfig[]>([])
    const root = ref<string>("")
    const allow = ref<boolean>((new Date()).getTime() / 1000 < 1740758400)

    return {
        loading_completed,
        member_table,
        members,
        select_members,
        user,
        groups,
        openai_list,
        auto_reply_rules,
        scheduled_rules,
        selectable,
        openai_config_list,
        root,
        allow
    }
})