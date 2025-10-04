<template>
  <div>
    <div class="toolbar">
      <el-button type="primary" :icon="Plus" @click="handleAdd">Add Job</el-button>
    </div>

    <el-table :data="jobs" border style="width: 100%">
      <el-table-column prop="id" label="ID" width="80" />
      <el-table-column prop="job_desc" label="Description" />
      <el-table-column prop="schedule_conf" label="Cron" />
      <el-table-column prop="author" label="Author" />
      <el-table-column label="Actions" width="200">
        <template #default="{ row }">
          <el-button size="small" @click="handleEdit(row)">Edit</el-button>
          <el-button size="small" type="danger" @click="handleDelete(row.id)">Delete</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog v-model="dialogVisible" :title="dialogTitle" width="60%">
      <el-form :model="form" label-width="150px">
        <el-form-item label="Job Group">
          <el-select v-model="form.job_group" placeholder="Select job group">
            <el-option
              v-for="group in jobGroups"
              :key="group.id"
              :label="group.title"
              :value="group.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="Description">
          <el-input v-model="form.job_desc" />
        </el-form-item>
        <el-form-item label="Author">
          <el-input v-model="form.author" />
        </el-form-item>
        <el-form-item label="Schedule Type">
          <el-input v-model="form.schedule_type" />
        </el-form-item>
        <el-form-item label="Schedule Conf">
          <el-input v-model="form.schedule_conf" placeholder="Cron expression" />
        </el-form-item>
        <el-form-item label="Executor Handler">
          <el-input v-model="form.executor_handler" />
        </el-form-item>
        <el-form-item label="Executor Param">
          <el-input v-model="form.executor_param" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">Cancel</el-button>
        <el-button type="primary" @click="handleSave">Save</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { Plus } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import api from '@/api'

const jobs = ref([])
const jobGroups = ref([])
const dialogVisible = ref(false)
const dialogTitle = ref('')
const form = ref({})

const fetchJobs = async () => {
  try {
    const response = await api.getJobs()
    jobs.value = response.data
  } catch (error) {
    ElMessage.error('Failed to fetch jobs.')
  }
}

const fetchJobGroups = async () => {
  try {
    const response = await api.getJobGroups()
    jobGroups.value = response.data
  } catch (error) {
    ElMessage.error('Failed to fetch job groups.')
  }
}

onMounted(() => {
  fetchJobs()
  fetchJobGroups()
})

const getNewForm = () => ({
  id: null,
  job_group: '',
  job_desc: '',
  author: '',
  schedule_type: 'CRON',
  schedule_conf: '',
  executor_handler: '',
  executor_param: '',
  // Add other fields with default values
  alarm_email: '',
  misfire_strategy: 'DO_NOTHING',
  executor_route_strategy: 'FIRST',
  executor_block_strategy: 'SERIAL_EXECUTION',
  executor_timeout: 0,
  executor_fail_retry_count: 0,
  glue_type: 'BEAN',
  glue_source: '',
  glue_remark: '',
  child_jobid: '',
  trigger_status: 0,
  trigger_last_time: 0,
  trigger_next_time: 0,
})

const handleAdd = () => {
  form.value = getNewForm()
  dialogTitle.value = 'Add Job'
  dialogVisible.value = true
}

const handleEdit = (row) => {
  form.value = { ...row }
  dialogTitle.value = 'Edit Job'
  dialogVisible.value = true
}

const handleSave = async () => {
  try {
    if (form.value.id) {
      await api.updateJob(form.value)
      ElMessage.success('Job updated successfully.')
    } else {
      await api.addJob(form.value)
      ElMessage.success('Job added successfully.')
    }
    dialogVisible.value = false
    await fetchJobs()
  } catch (error) {
    ElMessage.error('Failed to save job.')
  }
}

const handleDelete = (id) => {
  ElMessageBox.confirm('Are you sure you want to delete this job?', 'Warning', {
    confirmButtonText: 'OK',
    cancelButtonText: 'Cancel',
    type: 'warning',
  })
    .then(async () => {
      try {
        await api.deleteJob(id)
        ElMessage.success('Job deleted successfully.')
        await fetchJobs()
      } catch (error) {
        ElMessage.error('Failed to delete job.')
      }
    })
    .catch(() => {
      // Canceled
    })
}
</script>

<style scoped>
.toolbar {
  margin-bottom: 20px;
}
</style>