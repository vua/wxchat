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
    source: string,
    url: string,
    model: string[]
}

export interface ScheduledRule {
    Id: string
    Name: string
    Cron: string
    Group: Group
    Reply: Reply[]
    Status: boolean
}

export interface RedeemResp {
    ExpiredTime: number
    StatusInfo: StatusInfo,
}

export interface SwitchAutoReplyRuleStatusResp {
    Rules: AutoReplyRule[],
    StatusInfo: StatusInfo,
}

export interface StatusInfo {
    StatusCode: number,
    StatusMsg: string,
}

export const globalStore = defineStore('global', () => {
    const homepage_enable = ref<boolean>(false)
    const logout = ref<boolean>(false)
    const members = ref<Member[]>([])
    const select_members = ref<Member[]>([])
    const user = ref<User>({
        UserName: "",
        PYQuanPin: "",
        NickName: ""
    })
    const groups = ref<Group[]>([])
    const auto_reply_rules = ref<AutoReplyRule[]>([])
    const scheduled_rules = ref<ScheduledRule[]>([])
    const selectable = ref<boolean>(true)
    const member_table = ref<TableInstance>()
    const openai_list = ref<OpenAi[]>([])
    const openai_config_list = ref<OpenAiConfig[]>([])
    const root = ref<string>("")
    const expired_time = ref<number>(0)

    return {
        homepage_enable,
        logout,
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
        expired_time
    }
})