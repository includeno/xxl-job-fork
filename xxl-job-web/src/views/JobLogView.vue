<template>
  <div>
    <div class="toolbar">
      <el-select v-model="selectedJob" placeholder="Select a job to view logs" @change="fetchLogs">
        <el-option
          v-for="job in jobs"
          :key="job.id"
          :label="job.job_desc"
          :value="job.id"
        />
      </el-select>
    </div>

    <el-table :data="logs" border style="width: 100%">
      <el-table-column prop="id" label="Log ID" width="80" />
      <el-table-column prop="trigger_time" label="Trigger Time" />
      <el-table-column prop="trigger_code" label="Trigger Code" />
      <el-table-column prop="handle_time" label="Handle Time" />
      <el-table-column prop="handle_code" label="Handle Code" />
      <el-table-column prop="executor_address" label="Executor" />
      <el-table-column label="Actions" width="120">
        <template #default="{ row }">
          <el-button size="small" @click="viewLogDetails(row)">View Log</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog v-model="logDialogVisible" title="Log Details" width="70%">
      <pre>{{ selectedLog }}</pre>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import api from '@/api'

const jobs = ref([])
const logs = ref([])
const selectedJob = ref(null)
const logDialogVisible = ref(false)
const selectedLog = ref(null)

const fetchJobs = async () => {
  try {
    const response = await api.getJobs()
    jobs.value = response.data
  } catch (error) {
    ElMessage.error('Failed to fetch jobs.')
  }
}

const fetchLogs = async () => {
  if (!selectedJob.value) return
  try {
    const response = await api.getJobLogs(selectedJob.value)
    logs.value = response.data
  } catch (error) {
    ElMessage.error('Failed to fetch logs.')
  }
}

onMounted(fetchJobs)

const viewLogDetails = (log) => {
  selectedLog.value = JSON.stringify(log, null, 2)
  logDialogVisible.value = true
}
</script>

<style scoped>
.toolbar {
  margin-bottom: 20px;
}
pre {
  background-color: #f5f5f5;
  padding: 10px;
  border-radius: 4px;
}
</style>