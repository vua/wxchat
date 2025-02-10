<script setup lang="ts">
import {globalStore, Member} from "../stores/global_store.ts";
import {storeToRefs} from "pinia";
import {ref, computed} from "vue";
import {ElTable} from 'element-plus'
import {convertFileSrc} from "@tauri-apps/api/core";

const store = globalStore()
const {members, member_table, select_members, selectable, root} = storeToRefs(store)

const search = ref('')

const member_type = ref<string>("1")


const getMembers = computed(() =>
    members.value.filter(
        (data) => {
          if (member_type.value == "1" && (data.VerifyFlag != 0 || data.UserName.startsWith("@@"))) {
            return false
          }
          if (member_type.value == "2" && !data.UserName.startsWith("@@")) {
            return false
          }
          if (member_type.value == "3" && data.VerifyFlag == 0) {
            return false
          }
          return !search.value ||
              data.NickName.toLowerCase().includes(search.value.toLowerCase()) ||
              data.RemarkName.toLowerCase().includes(search.value.toLowerCase())
        }
    )
)

const handleSelectionChange = (val: Member[]) => {
  select_members.value = val
}

const getRowKey = (row: Member) => {
  return row.UserName
}

const getImgSrc = (name: string) => {
  return convertFileSrc(`${root.value}/resources/image/avatar/${name}.jpg`)
}

</script>

<template>
  <div class="member-container">
    <el-radio-group v-model="member_type"
                    style="background: rgb(246, 248, 250);padding: 10px 0;display: flex;justify-content: center">
      <el-radio value="1">好友</el-radio>
      <el-radio value="2">群聊</el-radio>
      <el-radio value="3">公众号</el-radio>
    </el-radio-group>
    <el-table
        ref="member_table"
        :row-key="getRowKey"
        :data="getMembers"
        highlight-current-row
        height="100vh"
        :header-cell-style="{background: 'rgb(246, 248, 250)'}"
        @selection-change="handleSelectionChange"
        class="concat-table"
    >
      <template #empty>
        <div style="background: rgb(246, 248, 250);width: 100%;height: 100%">
          暂无数据
        </div>
      </template>
      <el-table-column type="selection" width="30px" :reserve-selection="true" v-if="selectable"
                       class-name="concat-column"/>
      <el-table-column :label="member_type=='1'?'好友':(member_type=='2'?'群聊':'公众号')" class="concat-nickname"
                       show-overflow-tooltip class-name="concat-column">
        <template #default="scope">
          <div class="concat-nickname-container">
            <img class="avatar"
                 :src="getImgSrc(scope.row.PYQuanPin)"
                 alt="">
            <div class="name">
              <div v-html="scope.row.NickName"/>
              <div v-html="scope.row.RemarkName" style="margin-left:2px;color: #ccc;font-size: 12px"/>
            </div>
          </div>
        </template>
      </el-table-column>
      <el-table-column align="right" class="concat-search" class-name="concat-column">
        <template #header>
          <el-input v-model="search" size="small" placeholder="搜索"/>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>

<style scoped>

.member-container {
  width: 100%;
}

.concat-table {
  display: flex;
  justify-content: center;
}

.name {
  display: flex;
  flex-direction: row;
  justify-content: center;
  align-items: center;
}

.avatar {
  width: 30px;
  height: 30px;
  margin-right: 5px;
}

.concat-nickname-container {
  display: flex;
  flex-direction: row;
  width: 100%;
}

.concat-nickname {
  flex: 1;
}

.concat-search {
  flex: 1;
}

::v-deep .concat-column {
  background: rgb(246, 248, 250);
}

::v-deep .el-table__empty-block{
  background: rgb(246, 248, 250);
}

::v-deep .el-scrollbar{
  background: rgb(246, 248, 250);
}

</style>
